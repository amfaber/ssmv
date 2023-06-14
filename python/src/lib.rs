use super_simple_mesh_viewer::run_rust;
use numpy::{PyReadonlyArray2};
use pyo3::prelude::*;


#[pyfunction]
fn run(verts: PyReadonlyArray2<f32>, faces: PyReadonlyArray2<i32>) {

    let verts = verts.as_array().to_owned();
    let faces = faces.as_array().to_owned();
    
    run_rust(verts, faces);
}

#[pymodule]
fn ssmv(_py: Python<'_>, m: &PyModule) -> PyResult<()>{
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(()) 
}
