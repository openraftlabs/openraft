use actix::prelude::*;
use log::{debug, warn};

use crate::{
    AppError, NodeId,
    common::{DependencyAddr, UpdateCurrentLeader},
    messages::{VoteRequest, VoteResponse},
    network::RaftNetwork,
    raft::{RaftState, Raft},
    storage::RaftStorage,
};

impl<E: AppError, N: RaftNetwork<E>, S: RaftStorage<E>> Handler<VoteRequest> for Raft<E, N, S> {
    type Result = ResponseActFuture<Self, VoteResponse, ()>;

    /// An RPC invoked by candidates to gather votes (§5.2).
    ///
    /// Receiver implementation:
    ///
    /// 1. Reply `false` if `term` is less than receiver's current `term` (§5.1).
    /// 2. If receiver has not cast a vote for the current `term` or it voted for `candidate_id`, and
    ///    candidate’s log is atleast as up-to-date as receiver’s log, grant vote (§5.2, §5.4).
    fn handle(&mut self, msg: VoteRequest, ctx: &mut Self::Context) -> Self::Result {
        // Only handle requests if actor has finished initialization.
        if let &RaftState::Initializing = &self.state {
            warn!("Received Raft RPC before initialization was complete.");
            return Box::new(fut::err(()));
        }

        Box::new(fut::result(self._handle_vote_request(ctx, msg)))
    }
}

impl<E: AppError, N: RaftNetwork<E>, S: RaftStorage<E>> Raft<E, N, S> {
    /// Business logic of handling a `VoteRequest` RPC.
    fn _handle_vote_request(&mut self, ctx: &mut Context<Self>, msg: VoteRequest) -> Result<VoteResponse, ()> {
        // Don't interact with non-cluster members.
        if !self.members.contains(&msg.candidate_id) {
            return Err(());
        }
        debug!("Handling vote request on node {} from node {} for term {}.", &self.id, &msg.candidate_id, &msg.term);

        // If candidate's current term is less than this nodes current term, reject.
        if &msg.term < &self.current_term {
            return Ok(VoteResponse{term: self.current_term, vote_granted: false});
        }

        // If candidate's log is not at least as up-to-date as this node, then reject.
        if &msg.last_log_term < &self.last_log_term || &msg.last_log_index < &self.last_log_index {
            return Ok(VoteResponse{term: self.current_term, vote_granted: false});
        }

        // Candidate's log is up-to-date so handle voting conditions. //

        // If term is newer than current term, cast vote.
        if &msg.term > &self.current_term {
            self.current_term = msg.term;
            self.voted_for = Some(msg.candidate_id);
            self.save_hard_state(ctx);
            self.update_election_timeout(ctx);
            return Ok(VoteResponse{term: self.current_term, vote_granted: true});
        }

        // Term is the same as current term. This will be rare, but could come about from some error conditions.
        match &self.voted_for {
            // This node has already voted for the candidate.
            Some(candidate_id) if candidate_id == &msg.candidate_id => {
                self.update_election_timeout(ctx);
                Ok(VoteResponse{term: self.current_term, vote_granted: true})
            }
            // This node has already voted for a different candidate.
            Some(_) => Ok(VoteResponse{term: self.current_term, vote_granted: false}),
            // This node has not already voted, so vote for the candidate.
            None => {
                self.voted_for = Some(msg.candidate_id);
                self.save_hard_state(ctx);
                self.update_election_timeout(ctx);
                Ok(VoteResponse{term: self.current_term, vote_granted: true})
            },
        }
    }

    /// Request a vote from the the target peer.
    pub(super) fn request_vote(&mut self, _: &mut Context<Self>, target: NodeId) -> impl ActorFuture<Actor=Self, Item=(), Error=()> {
        let rpc = VoteRequest::new(target, self.current_term, self.id, self.last_log_index, self.last_log_term);
        fut::wrap_future(self.network.send(rpc))
            .map_err(|err, act: &mut Self, ctx| act.map_fatal_actix_messaging_error(ctx, err, DependencyAddr::RaftNetwork))
            .and_then(|res, _, _| fut::result(res))
            .and_then(|res, act, ctx| {
                // Ensure the node is still in candidate state.
                let state = match &mut act.state {
                    RaftState::Candidate(state) => state,
                    // If this node is not currently in candidate state, then this request is done.
                    _ => return fut::ok(()),
                };
                debug!("Node {} received request vote response. {:?}", &act.id, &res);

                // If peer's term is greater than current term, revert to follower state.
                if res.term > act.current_term {
                    act.become_follower(ctx);
                    act.current_term = res.term;
                    act.update_current_leader(ctx, UpdateCurrentLeader::Unknown);
                    act.save_hard_state(ctx);
                    return fut::ok(());
                }

                // If peer granted vote, then update campaign state.
                if res.vote_granted {
                    state.votes_granted += 1;
                    if state.votes_granted >= state.votes_needed {
                        // If the campaign was successful, go into leader state.
                        act.become_leader(ctx);
                    }
                }

                fut::ok(())
            })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// Unit Tests ////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix::prelude::*;
    use tempfile::tempdir_in;

    use crate::{
        Raft,
        config::Config, dev::*,
        memory_storage::{MemoryStorage},
        storage::RaftStorage,
    };

    #[test]
    fn test_request_vote() {
        // TODO:NOTE: not sure if we will keep these style of tests here.

        // Assemble //////////////////////////////////////////////////////////
        let sys = System::builder().stop_on_panic(true).name("test").build();
        let net = RaftRecorder::new().start();

        let dir = tempdir_in("/tmp").unwrap();
        let snapshot_dir = dir.path().to_string_lossy().to_string();

        let config = Config::build(snapshot_dir.clone()).metrics_rate(Duration::from_secs(1)).validate().unwrap();
        let memstore = MemoryStorage::new(vec![0], snapshot_dir).start();

        // Assert ////////////////////////////////////////////////////////////
        let test = Assert(Box::new(|act: &mut RaftRecorder, _| {
            let len = act.vote_requests().count();
            let first_req = act.vote_requests().nth(0).expect("Expected one vote request to be sent.");
            assert_eq!(len, 1, "Vote RPC count mismatch.");
            assert_eq!(first_req.target, 99, "Vote RPC target mismatch.");
            assert_eq!(first_req.term, 0, "Vote RPC term mismatch.");
            assert_eq!(first_req.candidate_id, 1000, "Vote RPC candidate_id mismatch.");
            assert_eq!(first_req.last_log_index, 0, "Vote RPC last_log_index mismatch.");
            assert_eq!(first_req.last_log_term, 0, "Vote RPC last_log_term mismatch.");

            System::current().stop();
            // TODO:NOTE: if this style of test needs to be explored more, we will need to assert
            // against the state of the Raft as it moves from state to state and handles events.
        }));

        // Action ////////////////////////////////////////////////////////////
        let _node_addr = Raft::create(move |ctx| {
            let mut inst = Raft::new(1000, config, net.clone(), memstore, net.clone().recipient());
            let f = inst.request_vote(ctx, 99)
                .then(move |_, _, _| {
                    net.do_send(test);
                    let res: Result<(), ()> = Ok(());
                    fut::result(res)
                });
            ctx.spawn(f);
            inst
        });

        let _ = sys.run();
    }
}