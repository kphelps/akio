use uuid::Uuid;

error_chain!{
    errors {
        ActorAlreadyExists(id: Uuid) {
            description("actor already exists")
            display("actor '{}' already exists", id)
        }
        ActorDestroyed
        InvalidActor(id: Uuid) {
            description("invalid actor")
            display("invalid actor: '{}'", id)
        }
    }
}
