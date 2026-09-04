use super::super::ImplSnapshot;

pub const ID: &str = "impl-exists";

pub fn value(snapshot: &ImplSnapshot) -> f64 {
    if snapshot.exists {
        1.0
    } else {
        0.0
    }
}
