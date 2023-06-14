use super_simple_mesh_viewer::run_rust;

#[pyfunction]
fn run(verts: PyReadonlyArray2<f32>, faces: PyReadonlyArray1<usize>) {

    let verts = verts.as_array().to_owned();
    let faces = faces.as_array().to_owned();
    
    run_rust(verts, faces);
}

#[pymodule]
fn ssmv(_py: Python<'_>, m: &PyModule) -> PyResult<()>{
    m.add_function(wrap_pyfunction!(run, m)?)?;
    Ok(()) 
}
