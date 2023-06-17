use std::{net::{TcpStream, ToSocketAddrs}, process::Stdio};

use super_simple_mesh_viewer::{run_rust, Message, Communication, Response, View, Vec3};
use numpy::{PyReadonlyArray2, PyArray, ndarray::Array, PyArray2, PyReadonlyArray1};
use pyo3::{prelude::*, types::PyTuple};



#[pyfunction]
fn run() {
    run_rust();
}

#[pyclass]
pub struct Connection{
    tcp: Option<TcpStream>,
}

impl Connection{
    fn connect(&mut self) -> &mut TcpStream{
        let addr = "localhost:6142".to_socket_addrs().unwrap().next().unwrap();
        let timeout = std::time::Duration::from_millis(50);
        let boot = std::time::Duration::from_secs(5);

        let tcp = match TcpStream::connect_timeout(&addr, timeout){
            Ok(stream) => {
                stream
            },
            Err(_) => {
                std::process::Command::new("python")
                    .arg("-c")
                    .arg("import ssmv; ssmv.run()")
                    .stdout(Stdio::null())
                    .spawn().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(20));
                TcpStream::connect_timeout(&addr, boot).unwrap()
                // message.send(&mut stream).unwrap();
            },
        };
        self.tcp = Some(tcp);
        self.tcp.as_mut().unwrap()
    }
    
    fn ensure_stream(& mut self) -> & mut TcpStream{
        if self.tcp.is_none(){
            self.connect();
        }
        self.tcp.as_mut().unwrap()
    }
}


#[pymethods]
impl Connection{
    
    #[new]
    fn new() -> PyResult<Self>{
        
        Ok(Self{ tcp: None })
    }

    fn set_view(&mut self, position: PyReadonlyArray1<f64>, look_at: PyReadonlyArray1<f64>){
        let stream = self.ensure_stream();
        let position = position.as_array();
        let look_at = look_at.as_array();
        let position = Vec3::new(position[0] as f32, position[1] as f32, position[2] as f32);
        let look_at = Vec3::new(look_at[0] as f32, look_at[1] as f32, look_at[2] as f32);
        let view = View{
            position,
            look_at,
        };
        Message::SetView(view).send(stream).unwrap();
    }

    fn request_view<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyTuple>{
        let stream = self.ensure_stream();
        Message::RequestView.send(stream).unwrap();
        let Response::GetView(View{
            position,
            look_at
        }) = Response::receive(stream).unwrap().unwrap() else {
            panic!("wrong response");
        };
        let position = vec![position.x, position.y, position.z];
        let look_at = vec![look_at.x, look_at.y, look_at.z];

        let position = PyArray::from_vec(py, position);
        let look_at = PyArray::from_vec(py, look_at);
        
        let tup = PyTuple::new(py, [position, look_at]);
        Ok(tup)
    }

    fn send(&mut self, verts: PyReadonlyArray2<f32>, faces: PyReadonlyArray2<i32>){
        let verts = verts.as_array().to_owned();
        let faces = faces.as_array().to_owned();
    
        let message = Message::Mesh { verts, faces };
        message.send(self.ensure_stream()).unwrap();
    }
    
    fn test<'py>(&mut self, py: Python<'py>) -> PyResult<()>{
        let n = 100;
        let nf = n as f32;
        let array = Array::from_shape_fn([n, n, n], |tup|{
            let indices = [tup.0 as f32, tup.1 as f32, tup.2 as f32,];
            let r2: f32 = indices.map(|idx| (idx - nf/2.).powi(2)).iter().sum();
            if r2 < (nf/3.).powi(2){
                1f32
            } else {
                0f32
            }
        });

        let array = PyArray::from_owned_array(py, array);
        let skimage = py.import("skimage.measure").unwrap();
        let res = skimage.getattr("marching_cubes").unwrap()
            .call1((array, )).unwrap();

        let verts = res.get_item(0).unwrap();
        let faces = res.get_item(1).unwrap();

        let verts_down = verts.downcast::<PyArray2<f32>>().unwrap().readonly();
        let faces_down = faces.downcast::<PyArray2<i32>>().unwrap().readonly();

        self.send(verts_down, faces_down);

        Ok(())
    }

}

#[pymodule]
fn ssmv(_py: Python<'_>, m: &PyModule) -> PyResult<()>{
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_class::<Connection>()?;
    Ok(()) 
}
