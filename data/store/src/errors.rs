error_chain! {
    foreign_links {
        BsonDecoderError(::bson::DecoderError);
        BsonEncoderError(::bson::EncoderError);
        BsonValueAccessError(::bson::ValueAccessError);
        MongoDB(::mongodb::Error);
    }

    errors {
        UnableToParseModel(id: String, msg: String) {
            description("unable to parse model"),
            display("unable to parse model with id '{}': {}", id, msg),
        }
    }
}
