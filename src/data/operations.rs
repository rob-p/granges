//! Implementations of various operations on data.
//!

use num_traits::{Float, ToPrimitive};
use std::iter::Sum;

/// Calculate the median.
///
/// This will clone and turn `numbers` into a `Vec`.
pub fn median<F: Float + Ord + Sum>(numbers: &[F]) -> F {
    let mut numbers = numbers.to_vec();
    numbers.sort_unstable();
    let mid = numbers.len() / 2;
    if numbers.len() % 2 == 0 {
        (numbers[mid - 1] + numbers[mid]) / F::from(2.0).unwrap()
    } else {
        numbers[mid]
    }
}

/// The (subset of) standard `bedtools map` operations.
pub enum Operation {
    Sum,
    Min,
    Max,
    Mean,
    Median,
    Collapse,
}

pub enum OperationResult<T>
where
    T: Float,
{
    Float(T),
    String(String),
}

pub fn float_compute<T>(operation: Operation, data: &[T]) -> Option<OperationResult<T>>
where
    T: Float + Sum<T> + ToPrimitive + Ord + Clone + ToString,
{
    match operation {
        Operation::Sum => {
            let sum: T = data.iter().cloned().sum();
            Some(OperationResult::Float(sum))
        }
        Operation::Min => data.iter().cloned().min().map(OperationResult::Float),
        Operation::Max => data.iter().cloned().max().map(OperationResult::Float),
        Operation::Mean => {
            if data.is_empty() {
                None
            } else {
                let sum: T = data.iter().cloned().sum();
                let mean = sum / T::from(data.len()).unwrap();
                Some(OperationResult::Float(mean))
            }
        }
        Operation::Median => Some(OperationResult::Float(median(data))),
        Operation::Collapse => {
            let collapsed = data
                .iter()
                .map(|num| num.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Some(OperationResult::String(collapsed))
        }
    }
}