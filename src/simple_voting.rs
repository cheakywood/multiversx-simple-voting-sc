#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem)]
pub struct PollOption<M: ManagedTypeApi> {
    name: ManagedBuffer<M>,
    vote_count: u64,
}

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct VoteCastEvent<M: ManagedTypeApi> {
    voter: ManagedAddress<M>,
    option_name: ManagedBuffer<M>,
}

#[multiversx_sc::contract]
pub trait VotingContract {
    #[init]
    fn init(
        &self,
        poll_question: ManagedBuffer<Self::Api>,
        options: ManagedVec<Self::Api, ManagedBuffer<Self::Api>>
    ) {
        require!(options.len() > 1, "There must be at least two options to vote on.");

        self.poll_question().set(poll_question);
        self.options().clear();

        for option in options.iter() {
            let poll_option = PollOption {
                name: option.clone_value(),
                vote_count: 0,
            };
            self.options().push(&poll_option);
        }
    }

    #[view(getPollQuestion)]
    fn get_poll_question(&self) -> ManagedBuffer<Self::Api> {
        self.poll_question().get()
    }

    #[view(getOptions)]
    fn get_options(&self) -> ManagedVec<Self::Api, PollOption<Self::Api>> {
        let mut result = ManagedVec::new();
        let options = self.options();

        for option in options.iter() {
            result.push(option);
        }

        result
    }

    #[endpoint(castVote)]
    fn cast_vote(&self, option_index: usize) {
        let caller = self.blockchain().get_caller();
        require!(!self.has_voted(&caller), "You have already voted.");

        let mut options = self.options();
        require!(option_index < options.len(), "Invalid option index.");

        let mut poll_option = options.get(option_index);
        poll_option.vote_count += 1;
        options.set(option_index, &poll_option);

        self.voted_addresses().insert(caller.clone());

        self.emit_vote_cast_event(VoteCastEvent {
            voter: caller,
            option_name: poll_option.name.clone(),
        });
    }

    #[event("vote_cast")]
    fn emit_vote_cast_event(&self, vote_event: VoteCastEvent<Self::Api>);

    // Storage
    #[storage_mapper("poll_question")]
    fn poll_question(&self) -> SingleValueMapper<Self::Api, ManagedBuffer<Self::Api>>;

    #[storage_mapper("options")]
    fn options(&self) -> VecMapper<Self::Api, PollOption<Self::Api>>;

    #[storage_mapper("voted_addresses")]
    fn voted_addresses(&self) -> UnorderedSetMapper<Self::Api, ManagedAddress<Self::Api>>;

    // Helper
    fn has_voted(&self, address: &ManagedAddress<Self::Api>) -> bool {
        self.voted_addresses().contains(address)
    }
}
