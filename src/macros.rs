/// Implements both `Discriminator` and `TryFrom<&[u8]>` for a given instruction data type.
macro_rules! impl_instruction_data {
    ($($data:ident => $instr:ident),* $(,)?) => {
        $(
            // Discriminator impl
            impl InstructionBytes for $data {
                const INSTRUCTION: AuctionInstruction = AuctionInstruction::$instr;
            }

            impl<'a> TryFrom<&'a [u8]> for $data
                where $data: bytemuck::Pod + bytemuck::Zeroable,
            {
                type Error = bytemuck::PodCastError;

                fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
                    bytemuck::try_pod_read_unaligned(data)
                }
            }

            impl<'a> InstructionData<'a> for $data {}
        )*
    };
}

pub(crate) use impl_instruction_data;
