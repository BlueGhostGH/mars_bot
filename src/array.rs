#![allow(unsafe_code)]

pub(crate) trait ArrayTransposeResult
{
    type Output;
    type Error;

    fn transpose_result(self) -> Result<Self::Output, Self::Error>;
}

impl<T, E, const N: usize> ArrayTransposeResult for [Result<T, E>; N]
{
    type Output = [T; N];
    type Error = E;

    fn transpose_result(self) -> Result<Self::Output, Self::Error>
    {
        let mut resulting_array = [const { ::core::mem::MaybeUninit::uninit() }; N];

        for (index, item) in self.into_iter().enumerate() {
            let _ = resulting_array[index].write(item?);
        }

        Ok(unsafe { ::core::mem::MaybeUninit::array_assume_init(resulting_array) })
    }
}
