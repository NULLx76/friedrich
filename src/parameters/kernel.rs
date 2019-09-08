//! Available kernels
//!
//! Derived from rusty-machines' [implementation](https://github.com/AtheMathmo/rusty-machine/blob/master/src/learning/toolkit/kernel.rs)
//! For more informations on the kernels and their usecase, see [Usual_covariance_functions](https://en.wikipedia.org/wiki/Gaussian_process#Usual_covariance_functions) and [kernel-functions-for-machine-learning-applications](http://crsouza.com/2010/03/17/kernel-functions-for-machine-learning-applications/#kernel_functions)

use std::ops::{Add, Mul};
use nalgebra::DMatrix;
use crate::matrix::RowVectorSlice;

//---------------------------------------------------------------------------------------
// TRAIT

/// The Kernel trait
///
/// Requires a function mapping two vectors to a scalar.
pub trait Kernel: Default
{
   /// Takes two equal length slices and returns a scalar.
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64;

   /// Optional, function that performs an automatic fit of the kernel parameters
   fn fit(&mut self, _training_inputs: &DMatrix<f64>, _training_outputs: &DMatrix<f64>) {}
}

//---------------------------------------------------------------------------------------
// FIT

/// use a formula adapted from [Silverman's rule of thumb](https://en.wikipedia.org/wiki/Kernel_density_estimation#A_rule-of-thumb_bandwidth_estimator) to have a guess at the bandwith
/// this might be too large if the distribution of the distances is not normal but is fast to compute, o(n²) of the number of samples
fn fit_bandwith_silverman(training_inputs: &DMatrix<f64>) -> f64
{
   // builds the sum of all distances between different samples
   let mut sum_distances = 0.;
   for (sample_index, sample) in training_inputs.row_iter().enumerate()
   {
      for sample2 in training_inputs.row_iter().skip(sample_index + 1)
      {
         let distance = (sample - sample2).norm();
         sum_distances += distance;
      }
   }

   // counts the number of distances that have been computed
   let nb_samples = training_inputs.nrows();
   let nb_distances = ((nb_samples * nb_samples - nb_samples) / 2) as f64;

   // computes silverman's formula for the univariate case
   let mean_distance = sum_distances / nb_distances;
   // 0.9 instead of 1.059 as recommended by silverman
   mean_distance * 0.9 * nb_distances.powf(0.2f64)
}

/// outputs the variance of the outputs as a best guess of the amplitude
///
/// when the output has several dimensions, use the mean of the variances of the columns
/// which might not make a lot of sense if they are not normalized
fn fit_amplitude(training_outputs: &DMatrix<f64>) -> f64
{
   training_outputs.column_variance().mean()
}

//---------------------------------------------------------------------------------------
// KERNEL COMBINAISON

/// The sum of two kernels
///
/// This struct should not be directly instantiated but instead
/// is created when we add two kernels together.
///
/// Note that it will be more efficient to implement the final kernel
/// manually yourself. However this provides an easy mechanism to test
/// different combinations.
#[derive(Debug)]
pub struct KernelSum<T, U>
   where T: Kernel,
         U: Kernel
{
   k1: T,
   k2: U
}

/// Computes the sum of the two associated kernels.
impl<T, U> Kernel for KernelSum<T, U>
   where T: Kernel,
         U: Kernel
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      self.k1.kernel(x1, x2) + self.k2.kernel(x1, x2)
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.k1.fit(training_inputs, training_outputs);
      self.k2.fit(training_inputs, training_outputs);
   }
}

impl<T: Kernel, U: Kernel> Default for KernelSum<T, U>
{
   fn default() -> Self
   {
      let k1 = T::default();
      let k2 = U::default();
      KernelSum { k1, k2 }
   }
}

/// The pointwise product of two kernels
///
/// This struct should not be directly instantiated but instead
/// is created when we multiply two kernels together.
///
/// Note that it will be more efficient to implement the final kernel
/// manually yourself. However this provides an easy mechanism to test
/// different combinations.
#[derive(Debug)]
pub struct KernelProd<T, U>
   where T: Kernel,
         U: Kernel
{
   k1: T,
   k2: U
}

/// Computes the product of the two associated kernels.
impl<T, U> Kernel for KernelProd<T, U>
   where T: Kernel,
         U: Kernel
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      self.k1.kernel(x1, x2) * self.k2.kernel(x1, x2)
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.k1.fit(training_inputs, training_outputs);
      self.k2.fit(training_inputs, training_outputs);
   }
}

impl<T: Kernel, U: Kernel> Default for KernelProd<T, U>
{
   fn default() -> Self
   {
      let k1 = T::default();
      let k2 = U::default();
      KernelProd { k1, k2 }
   }
}

/// A wrapper tuple struct used for kernel arithmetic
#[derive(Debug)]
pub struct KernelArith<K: Kernel>(pub K);

impl<T: Kernel, U: Kernel> Add<KernelArith<T>> for KernelArith<U>
{
   type Output = KernelSum<U, T>;

   fn add(self, ker: KernelArith<T>) -> KernelSum<U, T>
   {
      KernelSum { k1: self.0, k2: ker.0 }
   }
}

impl<T: Kernel, U: Kernel> Mul<KernelArith<T>> for KernelArith<U>
{
   type Output = KernelProd<U, T>;

   fn mul(self, ker: KernelArith<T>) -> KernelProd<U, T>
   {
      KernelProd { k1: self.0, k2: ker.0 }
   }
}

//---------------------------------------------------------------------------------------
// CLASSICAL KERNELS

/// The Linear Kernel
///
/// k(x,y) = x^Ty + c
#[derive(Clone, Copy, Debug)]
pub struct Linear
{
   /// Constant term added to inner product.
   pub c: f64
}

impl Linear
{
   /// Constructs a new Linear Kernel.
   ///
   /// # Examples
   pub fn new(c: f64) -> Linear
   {
      Linear { c: c }
   }
}

/// Constructs the default Linear Kernel
///
/// The defaults are:
///
/// - c = 0
impl Default for Linear
{
   fn default() -> Linear
   {
      Linear { c: 0f64 }
   }
}

impl Kernel for Linear
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      x1.dot(&x2) + self.c
   }
}

//-----------------------------------------------

/// The Polynomial Kernel
///
/// k(x,y) = (αx^Ty + c)^d
#[derive(Clone, Copy, Debug)]
pub struct Polynomial
{
   /// Scaling of the inner product.
   pub alpha: f64,
   /// Constant added to inner product.
   pub c: f64,
   /// The power to raise the sum to.
   pub d: f64
}

impl Polynomial
{
   /// Constructs a new Polynomial Kernel.
   pub fn new(alpha: f64, c: f64, d: f64) -> Polynomial
   {
      Polynomial { alpha: alpha, c: c, d: d }
   }
}

/// Construct a new polynomial kernel.
///
/// The defaults are:
///
/// - alpha = 1
/// - c = 0
/// - d = 1
impl Default for Polynomial
{
   fn default() -> Polynomial
   {
      Polynomial { alpha: 1f64, c: 0f64, d: 1f64 }
   }
}

impl Kernel for Polynomial
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      (self.alpha * x1.dot(&x2) + self.c).powf(self.d)
   }
}

//-----------------------------------------------

/// Gaussian kernel
///
/// Equivalently a squared exponential kernel.
///
/// k(x,y) = A exp(-||x-y||² / 2l²)
///
/// Where A is the amplitude and l the length scale.
pub type Gaussian = SquaredExp;

/// Squared exponential kernel
///
/// Equivalently a gaussian function.
///
/// k(x,y) = A exp(-||x-y||² / 2l²)
///
/// Where A is the amplitude and l the length scale.
#[derive(Clone, Copy, Debug)]
pub struct SquaredExp
{
   /// The length scale of the kernel.
   pub ls: f64,
   /// The amplitude of the kernel.
   pub ampl: f64
}

impl SquaredExp
{
   /// Construct a new squared exponential kernel (gaussian).
   pub fn new(ls: f64, ampl: f64) -> SquaredExp
   {
      SquaredExp { ls: ls, ampl: ampl }
   }
}

/// Constructs the default Squared Exp kernel.
///
/// The defaults are:
///
/// - ls = 1
/// - ampl = 1
impl Default for SquaredExp
{
   fn default() -> SquaredExp
   {
      SquaredExp { ls: 1f64, ampl: 1f64 }
   }
}

impl Kernel for SquaredExp
{
   /// The squared exponential kernel function.
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      let distance_squared = (x1 - x2).norm_squared();
      let x = -distance_squared / (2f64 * self.ls * self.ls);
      self.ampl * x.exp()
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.ls = fit_bandwith_silverman(training_inputs);
      self.ampl = fit_amplitude(training_outputs);
   }
}

//-----------------------------------------------

/// The Exponential Kernel
///
/// k(x,y) = A exp(-||x-y|| / 2l²)
///
/// Where A is the amplitude and l is the length scale.
#[derive(Clone, Copy, Debug)]
pub struct Exponential
{
   /// The length scale of the kernel.
   pub ls: f64,
   /// The amplitude of the kernel.
   pub ampl: f64
}

impl Exponential
{
   /// Construct a new squared exponential kernel.
   pub fn new(ls: f64, ampl: f64) -> Exponential
   {
      Exponential { ls: ls, ampl: ampl }
   }
}

/// Constructs the default Exponential kernel.
///
/// The defaults are:
///
/// - ls = 1
/// - amplitude = 1
impl Default for Exponential
{
   fn default() -> Exponential
   {
      Exponential { ls: 1f64, ampl: 1f64 }
   }
}

impl Kernel for Exponential
{
   /// The squared exponential kernel function.
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      let distance = (x1 - x2).norm();
      let x = -distance / (2f64 * self.ls * self.ls);
      self.ampl * x.exp()
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.ls = fit_bandwith_silverman(training_inputs);
      self.ampl = fit_amplitude(training_outputs);
   }
}

//-----------------------------------------------

/// The Matèrn1 kernel which is 1 differentiable and correspond to a classical Matèrn kernel with nu=3/2
///
/// k(x,y) = A (1 + ||x-y||sqrt(3)/l) exp(-||x-y||sqrt(3)/l)
///
/// Where A is the amplitude and l is the length scale.
#[derive(Clone, Copy, Debug)]
pub struct Matern1
{
   /// The length scale of the kernel.
   pub ls: f64,
   /// The amplitude of the kernel.
   pub ampl: f64
}

impl Matern1
{
   /// Construct a new matèrn1 kernel.
   pub fn new(ls: f64, ampl: f64) -> Matern1
   {
      Matern1 { ls: ls, ampl: ampl }
   }
}

/// Constructs the default Matern1 kernel.
///
/// The defaults are:
///
/// - ls = 1
/// - amplitude = 1
impl Default for Matern1
{
   fn default() -> Matern1
   {
      Matern1 { ls: 1f64, ampl: 1f64 }
   }
}

impl Kernel for Matern1
{
   /// The matèrn1 kernel function.
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      let distance = (x1 - x2).norm();
      let x = (3f64).sqrt() * distance / self.ls;
      self.ampl * (1f64 + x) * (-x).exp()
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.ls = fit_bandwith_silverman(training_inputs);
      self.ampl = fit_amplitude(training_outputs);
   }
}

//-----------------------------------------------

/// The Matèrn2 kernel which is 2 differentiable and correspond to a classical Matèrn kernel with nu=5/2
///
/// k(x,y) = A (1 + ||x-y||sqrt(5)/l + ||x-y||²5/3l²) exp(-||x-y||sqrt(5)/l)
///
/// Where A is the amplitude and l is the length scale.
#[derive(Clone, Copy, Debug)]
pub struct Matern2
{
   /// The length scale of the kernel.
   pub ls: f64,
   /// The amplitude of the kernel.
   pub ampl: f64
}

impl Matern2
{
   /// Construct a new matèrn2 kernel.
   pub fn new(ls: f64, ampl: f64) -> Matern2
   {
      Matern2 { ls: ls, ampl: ampl }
   }
}

/// Constructs the default Matern2 kernel.
///
/// The defaults are:
///
/// - ls = 1
/// - amplitude = 1
impl Default for Matern2
{
   fn default() -> Matern2
   {
      Matern2 { ls: 1f64, ampl: 1f64 }
   }
}

impl Kernel for Matern2
{
   /// The matèrn2 kernel function.
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      let distance = (x1 - x2).norm();
      let x = (5f64).sqrt() * distance / self.ls;
      self.ampl * (1f64 + x + (5f64 * distance * distance) / (3f64 * self.ls * self.ls)) * (-x).exp()
   }

   fn fit(&mut self, training_inputs: &DMatrix<f64>, training_outputs: &DMatrix<f64>)
   {
      self.ls = fit_bandwith_silverman(training_inputs);
      self.ampl = fit_amplitude(training_outputs);
   }
}

//-----------------------------------------------

/// The Hyperbolic Tangent Kernel.
///
/// ker(x,y) = _tanh_(αx^Ty + c)
#[derive(Clone, Copy, Debug)]
pub struct HyperTan
{
   /// The scaling of the inner product.
   pub alpha: f64,
   /// The constant to add to the inner product.
   pub c: f64
}

impl HyperTan
{
   /// Constructs a new Hyperbolic Tangent Kernel.
   pub fn new(alpha: f64, c: f64) -> HyperTan
   {
      HyperTan { alpha: alpha, c: c }
   }
}

/// Constructs a default Hyperbolic Tangent Kernel.
///
/// The defaults are:
///
/// - alpha = 1
/// - c = 0
impl Default for HyperTan
{
   fn default() -> HyperTan
   {
      HyperTan { alpha: 1f64, c: 0f64 }
   }
}

impl Kernel for HyperTan
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      (self.alpha * x1.dot(&x2) + self.c).tanh()
   }
}

//-----------------------------------------------

/// The Multiquadric Kernel.
///
/// k(x,y) = sqrt(||x-y||² + c²)
#[derive(Clone, Copy, Debug)]
pub struct Multiquadric
{
   /// Constant added to square of difference.
   pub c: f64
}

impl Multiquadric
{
   /// Constructs a new Multiquadric Kernel.
   pub fn new(c: f64) -> Multiquadric
   {
      Multiquadric { c: c }
   }
}

/// Constructs a default Multiquadric Kernel.
///
/// The defaults are:
///
/// - c = 0
impl Default for Multiquadric
{
   fn default() -> Multiquadric
   {
      Multiquadric { c: 0f64 }
   }
}

impl Kernel for Multiquadric
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      (x1 - x2).norm_squared().hypot(self.c)
   }
}

//-----------------------------------------------

/// The Rational Quadratic Kernel.
///
/// k(x,y) = (1 + ||x-y||² / (2αl²))^-α
#[derive(Clone, Copy, Debug)]
pub struct RationalQuadratic
{
   /// Controls inverse power and difference scale.
   pub alpha: f64,
   /// Length scale controls scale of difference.
   pub ls: f64
}

impl RationalQuadratic
{
   /// Constructs a new Rational Quadratic Kernel.
   pub fn new(alpha: f64, ls: f64) -> RationalQuadratic
   {
      RationalQuadratic { alpha: alpha, ls: ls }
   }
}

/// The default Rational Qaudratic Kernel.
///
/// The defaults are:
///
/// - alpha = 1
/// - ls = 1
impl Default for RationalQuadratic
{
   fn default() -> RationalQuadratic
   {
      RationalQuadratic { alpha: 1f64, ls: 1f64 }
   }
}

impl Kernel for RationalQuadratic
{
   fn kernel(&self, x1: RowVectorSlice, x2: RowVectorSlice) -> f64
   {
      let distance_squared = (x1 - x2).norm_squared();
      (1f64 + distance_squared / (2f64 * self.alpha * self.ls * self.ls)).powf(-self.alpha)
   }
}