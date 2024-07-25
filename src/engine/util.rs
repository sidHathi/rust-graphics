use cgmath::Quaternion;
use rand::Rng;

/// Generate a random Quaternion
pub fn random_quaternion() -> Quaternion<f32> {
    let mut rng = rand::thread_rng();
    let u1: f32 = rng.gen();
    let u2: f32 = rng.gen();
    let u3: f32 = rng.gen();

    let q1 = (1.0 - u1).sqrt() * (2.0 * std::f32::consts::PI * u2).sin();
    let q2 = (1.0 - u1).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
    let q3 = u1.sqrt() * (2.0 * std::f32::consts::PI * u3).sin();
    let q4 = u1.sqrt() * (2.0 * std::f32::consts::PI * u3).cos();

    Quaternion::new(q1, q2, q3, q4)
}