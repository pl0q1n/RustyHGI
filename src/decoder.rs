use utils::Metadata;

trait Decoder {
    type Input;
    type Output;

    fn decode(&mut self, metadata: Metadata, input: Self::Input) -> Self::Output;
}
