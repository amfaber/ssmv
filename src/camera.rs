use smooth_bevy_cameras::{LookAngles, LookTransform, LookTransformBundle, Smoother};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    time::Time,
    transform::components::Transform,
};

#[derive(Default)]
pub struct MyCameraPlugin {
    pub override_input_system: bool,
}

// impl MyCameraPlugin {
//     pub fn new(override_input_system: bool) -> Self {
//         Self {
//             override_input_system,
//         }
//     }
// }

impl Plugin for MyCameraPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            // .add_system(on_controller_enabled_changed.in_base_set(CoreSet::PreUpdate))
            .add_system(control_system)
            .add_event::<ControlEvent>();
        if !self.override_input_system {
            app.add_system(default_input_map);
        }
    }
}

#[derive(Bundle)]
pub struct MyCameraBundle {
    controller: MyCameraController,
    #[bundle]
    look_transform: LookTransformBundle,
    transform: Transform,
}

impl MyCameraBundle {
    pub fn new(controller: MyCameraController, eye: Vec3, target: Vec3, up: Vec3) -> Self {
        // Make sure the transform is consistent with the controller to start.
        let transform = Transform::from_translation(eye).looking_at(target, up);

        Self {
            controller,
            look_transform: LookTransformBundle {
                transform: LookTransform::new(eye, target, up),
                smoother: Smoother::new(controller.smoothing_weight),
            },
            transform,
        }
    }
}

/// A camera controlled with the mouse in the same way as My Engine's viewport controller.
#[derive(Clone, Component, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MyCameraController {
    /// Whether to process input or ignore it
    pub enabled: bool,

    /// How many radians per frame for each rotation axis (yaw, pitch) when rotating with the mouse
    pub rotate_sensitivity: Vec2,

    /// How many units per frame for each direction when translating using Middle or L+R panning
    pub mouse_translate_sensitivity: Vec2,

    /// How many units per frame when translating using scroll wheel
    pub wheel_translate_sensitivity: f32,

    /// How many units per frame when translating using W/S/Q/E
    /// Updated with scroll wheel while dragging with any mouse button
    pub keyboard_mvmt_sensitivity: f32,

    /// Wheel sensitivity for modulating keyboard movement speed
    pub keyboard_mvmt_wheel_sensitivity: f32,

    /// The greater, the slower to follow input
    pub smoothing_weight: f32,

    pub pixels_per_line: f32,
    pub mouse_wheel_zoom_sensitivity: f32,

    // pub grid: ,
}

impl Default for MyCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            rotate_sensitivity: Vec2::splat(0.2),
            mouse_translate_sensitivity: Vec2::splat(2.0),
            wheel_translate_sensitivity: 50.0,
            keyboard_mvmt_sensitivity: 100.0,
            keyboard_mvmt_wheel_sensitivity: 5.0,
            smoothing_weight: 0.7,
            pixels_per_line: 53.0,
            mouse_wheel_zoom_sensitivity: 0.1,
        }
    }
}

#[derive(Debug)]
pub enum ControlEvent {
    Locomotion { translation: Vec3, rotation: Vec2 },
    Orbit { rotation: Vec2, zoom: f32 },
    // Rotate(Vec2),
    // TranslateEye(Vec3),
}

pub fn default_input_map(
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    keyboard: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut controllers: Query<&mut MyCameraController>,
) {
    // Can only control one camera at a time.
    let mut controller = if let Some(controller) = controllers.iter_mut().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let MyCameraController {
        rotate_sensitivity: mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        wheel_translate_sensitivity,
        mut keyboard_mvmt_sensitivity,
        keyboard_mvmt_wheel_sensitivity,
        ..
    } = *controller;

    let left_pressed = mouse_buttons.pressed(MouseButton::Left);
    let right_pressed = mouse_buttons.pressed(MouseButton::Right);
    let middle_pressed = mouse_buttons.pressed(MouseButton::Middle);

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    let mut wheel_delta = 0.0;
    for event in mouse_wheel_reader.iter() {
        wheel_delta += match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / controller.pixels_per_line,
        };
    }

    // let mut panning_dir = Vec2::ZERO;
    // let mut translation_dir = Vec2::ZERO; // y is forward/backward axis, x is rotation around Z

    // Relative to the cameras coordinates.
    // x = left/right
    // y = down/up
    // z = backward/forward
    let mut locomotion_dir = Vec3::ZERO;

    for key in keyboard.get_pressed() {
        match key {
            KeyCode::A => {
                locomotion_dir.x -= 1.0;
            }

            KeyCode::D => {
                locomotion_dir.x += 1.0;
            }

            KeyCode::Q => {
                locomotion_dir.y -= 1.0;
            }

            KeyCode::E => {
                locomotion_dir.y += 1.0;
            }

            KeyCode::S => {
                locomotion_dir.z -= 1.0;
            }

            KeyCode::W => {
                locomotion_dir.z += 1.0;
            }

            _ => {}
        }
    }

    let mut locomotion = Vec3::ZERO;

    // If any of the mouse button are pressed; read additional signals from the keyboard for panning
    // and locomotion along camera view axis

    // let mut zoom = 1.0;
    // for event in mouse_wheel_reader.iter() {
    //     // scale the event magnitude per pixel or per line
    //     let scroll_amount = match event.unit {
    //         MouseScrollUnit::Line => event.y,
    //         MouseScrollUnit::Pixel => event.y / controller.pixels_per_line,
    //     };
    //     dbg!(scroll_amount);
    //     zoom *= 1.0 - scroll_amount * controller.mouse_wheel_zoom_sensitivity;
    // }

    if right_pressed || keyboard.pressed(KeyCode::LShift) || keyboard.pressed(KeyCode::RShift) {
        locomotion += keyboard_mvmt_sensitivity * locomotion_dir;
        keyboard_mvmt_sensitivity += keyboard_mvmt_wheel_sensitivity * wheel_delta;
        controller.keyboard_mvmt_sensitivity = keyboard_mvmt_sensitivity.max(0.01);
        events.send(ControlEvent::Locomotion {
            translation: locomotion,
            rotation: mouse_rotate_sensitivity * cursor_delta,
        });
    } else if left_pressed {
        let rotation = mouse_rotate_sensitivity * cursor_delta;
        let zoom = 1.0 - wheel_delta * controller.mouse_wheel_zoom_sensitivity;
        // let mut zoom = 1.0;
        // for event in mouse_wheel_reader.iter() {
        //     // scale the event magnitude per pixel or per line
        //     let scroll_amount = match event.unit {
        //         MouseScrollUnit::Line => event.y,
        //         MouseScrollUnit::Pixel => event.y / controller.pixels_per_line,
        //     };
        //     dbg!(scroll_amount);
        //     zoom *= 1.0 - scroll_amount * controller.mouse_wheel_zoom_sensitivity;
        // }

        events.send(ControlEvent::Orbit { rotation, zoom });
    }

    // You can also pan using the mouse only; add those signals to existing panning
    // if middle_pressed || (left_pressed && right_pressed) {
    //     panning += mouse_translate_sensitivity * cursor_delta;
    // }

    // When left only is pressed, mouse movements add up to the "Unreal locomotion" scheme
    // if left_pressed && !middle_pressed && !right_pressed {
    //     locomotion.x = mouse_rotate_sensitivity.x * cursor_delta.x;
    //     locomotion.y -= mouse_translate_sensitivity.y * cursor_delta.y;
    // }

    // if panning.length_squared() > 0.0 {
    //     events.send(ControlEvent::TranslateEye(panning));
    // }

    // if locomotion.length_squared() > 0.0 {
    //     events.send(ControlEvent::Locomotion(locomotion));
    // }
}

pub fn control_system(
    time: Res<Time>,
    mut events: EventReader<ControlEvent>,
    mut cameras: Query<(&MyCameraController, &mut LookTransform)>,
) {
    // Can only control one camera at a time.
    let mut transform = if let Some((_, transform)) = cameras.iter_mut().find(|c| c.0.enabled) {
        transform
    } else {
        return;
    };

    let look_vector = match transform.look_direction() {
        Some(safe_look_vector) => safe_look_vector,
        None => return,
    };

    let dt = time.delta_seconds();
    for event in events.iter() {
        match event {
            // ControlEvent::Locomotion(delta) => {
            //     // Translates forward/backward and rotates about the Y axis.
            //     look_angles.add_yaw(dt * -delta.x);
            //     dbg!(delta.x);
            //     transform.eye += dt * delta.y * look_vector;
            // }
            ControlEvent::Locomotion {
                translation,
                rotation,
            } => {
                let mut look_angles = LookAngles::from_vector(look_vector);
                let radius = transform.radius();
                // Rotates with pitch and yaw.
                look_angles.add_yaw(dt * -rotation.x);
                look_angles.add_pitch(dt * -rotation.y);

                let yaw_rot = Quat::from_axis_angle(Vec3::Y, look_angles.get_yaw());
                let left_right = yaw_rot * Vec3::X;

                // let pitch_rot = Quat::from_axis_angle(Vec3::X, look_angles.get_pitch());
                // let rot_y = pitch_rot * yaw_rot * Vec3::Y;
                let back_forward = look_vector;
                let down_up = back_forward.cross(left_right);

                // Translates up/down (Y) and left/right (X).
                // transform.eye += dt * -delta.x * rot_x + Vec3::new(0.0, dt * delta.y, 0.0);
                transform.eye += dt * -translation.x * left_right
                    + dt * translation.y * down_up
                    + dt * back_forward * translation.z;

                transform.target = transform.eye + radius * look_angles.unit_vector();
            }

            ControlEvent::Orbit { rotation, zoom } => {
                let mut look_angles = LookAngles::from_vector(-look_vector);
                look_angles.add_yaw(dt * -rotation.x);
                look_angles.add_pitch(dt * rotation.y);
                // dbg!(zoom);

                let new_radius = (zoom * transform.radius()).min(1000000.0).max(0.001);
                transform.eye = transform.target + new_radius * look_angles.unit_vector();
            }
        }
    }

    // look_angles.assert_not_looking_up();
}
