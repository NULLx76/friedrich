#![allow(dead_code)]

mod multivariate_normal;
mod parameters;
mod gaussian_process;
mod matrix;

use nalgebra::{DMatrix};
use crate::gaussian_process::GaussianProcess;

fn main()
{
   // training data
   let training_inputs = DMatrix::from_column_slice(4, 1, &[0.8, 1.2, 3.8, 4.2]);
   let training_outputs = DMatrix::from_column_slice(4, 1, &[3.0, 4.0, -2.0, -2.0]);

   // builds a model
   //let gp = GaussianProcess::default(training_inputs, training_outputs);
   let gp = GaussianProcess::default(training_inputs, training_outputs);

   // make a prediction on new data
   let inputs = DMatrix::from_column_slice(5, 1, &[1.0, 2.0, 3.0, 4.2, 7.]);
   let outputs = gp.predict_mean(&inputs);
   println!("prediction: {}", outputs);
   let sd = gp.predict_standard_deviation(&inputs);
   println!("standard deviation: {}", sd);

   // sample the gaussian process on new data
   let sampler = gp.sample_at(&inputs);
   let mut rng = rand::thread_rng();
   for i in 1..=5
   {
      println!("sample {}: {}", i, sampler.sample(&mut rng));
   }
}
