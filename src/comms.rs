use bevy::prelude::Vec3;
use ndarray::Array2;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};

pub fn listen(sender: Sender<Message>, receiver: Receiver<Response>){
    let listener = TcpListener::bind("localhost:6142").unwrap();
    loop{
        match listener.accept(){
            Ok((mut stream, _addr)) => {
                loop{
                    let recv = Message::receive(&mut stream);
                    match recv.unwrap(){
                        Some(message) => {
							let should_respond = message.requires_response();
							sender.send(message).unwrap();
							if should_respond{
								let response = receiver.recv().unwrap();
								response.send(&mut stream).unwrap();
							}
						},
                        None => break
                    }
                }
                // handle_client(stream, &sender)
            },
            Err(_e) => eprintln!("errored"),
        }
    }
}

pub trait Communication: Serialize + DeserializeOwned + Send{
    fn send(&self, stream: &mut TcpStream) -> anyhow::Result<()>{
        let bytes = bincode::serialize(self)?;
        let len = usize::to_le_bytes(bytes.len());
        stream.write_all(&len)?;
        stream.write_all(&bytes)?;

        Ok(())
    }

    fn receive(stream: &mut TcpStream) -> anyhow::Result<Option<Self>>{
        let mut len = [0; std::mem::size_of::<usize>()];
        let n_read = stream.read(&mut len)?;
        if n_read == 0{
            return Ok(None)
        }
        let len = usize::from_le_bytes(len);
		dbg!(len);
        let mut message = vec![0; len];
        stream.read(&mut message)?;
        let message: Self = bincode::deserialize(&message)?;
        Ok(Some(message))
    }
}

impl<T: Serialize + DeserializeOwned + Send> Communication for T {}

#[derive(Serialize, Deserialize, Debug)]
pub struct View{
	pub position: Vec3,
	pub look_at: Vec3,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message{
    Mesh{
        verts: Array2<f32>,
        faces: Array2<i32>,
    },
	SetView(View),
	RequestView,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response{
	GetView(View),
	Other,
}


impl Message{
	fn requires_response(&self) -> bool{
		match self{
		    Self::Mesh { .. } => false,
			Self::SetView(_) => false,
			Self::RequestView => true,
		}
	}
}

