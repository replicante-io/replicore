error_chain! {
    foreign_links {
        BsonDecoderError(::bson::DecoderError);
        BsonEncoderError(::bson::EncoderError);
        BsonValueAccessError(::bson::ValueAccessError);
        MongoDB(::mongodb::Error);
    }
}
