error_chain! {
    foreign_links {
        BsonDecoderError(::bson::DecoderError);
        BsonEncoderError(::bson::EncoderError);
        MongoDB(::mongodb::Error);
    }
}
