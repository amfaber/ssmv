use ndarray::array;
use super_simple_mesh_viewer::run_rust;

fn main(){
    let verts = array![
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
    ];
    let faces = array![
        [0, 1, 2,]
    ];

    run_rust(verts, faces);
}
