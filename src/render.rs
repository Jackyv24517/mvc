use bevy::{
    core_pipeline::Skybox,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{TextureViewDescriptor, TextureViewDimension},
    },
    window::{WindowRef, WindowResolution},
};

use crate::{
    components::{ModelRotateReposition, OrbitCamera},
    resource::{OperationSettings, OperationWindowRelatedEntities, SkyboxAttribute},
    states::OperationState,
};

// this will be the function responsible to spawn a window for the 3d model to render in
pub fn interactive(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    model_path: String,
    mut images: ResMut<Assets<Image>>,
    skybox_attributes: Res<SkyboxAttribute>,
    mut operation_window: ResMut<OperationWindowRelatedEntities>,
    operation_settings: Res<OperationSettings>,
) {
    // spawn a new window ( In MVC, there will be a maximum of 2 window at the same time, 1 for MVC main menu and the other will be for 3d model )
    let interac_window = commands
        .spawn(Window {
            title: "Interactive 📱".to_string(),
            resolution: WindowResolution::new(1280., 720.),
            position: WindowPosition::At(IVec2::new(300, 0)),
            ..default()
        })
        .id();

    let image = images.get_mut(skybox_attributes.skybox_handler.as_ref().unwrap());
    if let Some(image) = image {
        // get the loaded image back and process it so that it can be compatible for a 3d dimension
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array((image.height() / image.width()).max(1));
            print!("{}", image.texture_descriptor.array_layer_count());
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),

                ..Default::default()
            });
        }
    }

    // Spawn the camera for the 3d model window
    let interac_window_camera = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.0, operation_settings.radius_start_position)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            EnvironmentMapLight {
                diffuse_map: asset_server
                    .load("embedded://mvc/assets/pisa_diffuse_rgb9e5_zstd.ktx2"), // load the environment map light from embedded resource
                specular_map: asset_server
                    .load("embedded://mvc/assets/pisa_diffuse_rgb9e5_zstd.ktx2"), // load the environment map light from embedded resource
                intensity: 250.0,
                ..default()
            },
            // IMPORTANT, we need to tell the camera where to target
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(interac_window)),
                hdr: true,
                ..default()
            },
            Skybox {
                image: skybox_attributes.skybox_handler.as_ref().unwrap().clone(),
                brightness: 1000.0,
                ..default()
            },
        ))
        // this will be relavent for use to control the orbiting of the model
        .insert(OrbitCamera {
            window: interac_window,
            radius: operation_settings.radius_start_position,
            yaw: 0.0,
            pitch: 0.0,
            is_dragging: false,
        })
        .insert(Transform::from_scale(Vec3::new(0.5, 0.5, 0.5)))
        .id();

    // directional light
    let directional_light = commands
        .spawn((DirectionalLight {
            illuminance: 300.0,
            shadows_enabled: false,
            ..default()
        },))
        .id();

    // spawn the 3d model
    let scene_entity = commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path.clone()))),
            Transform::from_translation(Vec3::ZERO),
            ModelRotateReposition {
                window: interac_window,
                x: 0.0,
                y: 0.0,
            },
        ))
        .id();

    let node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(12.0),
        left: Val::Px(12.0),
        ..default()
    };

    let node_entity = commands
        .spawn((node, TargetCamera(interac_window_camera)))
        .id();

    let entities_list: Vec<Entity> = vec![
        interac_window,
        // skybox_entity,
        interac_window_camera,
        directional_light,
        scene_entity,
        node_entity,
    ];

    // saving the entites to a list, so that we can easily despawn them when the window close
    operation_window.window = Some(interac_window);
    operation_window.entities_list = Some(entities_list)
}
// set the ambient light that is used for the scene
pub fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 300.0;
}

// to reposition the model on the 3D environment
pub fn reposition_rotate_model(
    mut query: Query<(&mut Transform, &mut ModelRotateReposition)>,
    keys: Res<ButtonInput<KeyCode>>,
    current_operation_state: Res<State<OperationState>>,
    operation_window: ResMut<OperationWindowRelatedEntities>,
    operation_settings: Res<OperationSettings>,
) {
    let model_query = query.get_single_mut();
    match model_query {
        Ok((mut transform, mut model)) => {
            if model.window == operation_window.window.unwrap() {
                let c_o_s = current_operation_state.as_ref().get();
                if *c_o_s == OperationState::Interactive {
                    // rotate model
                    if keys.pressed(KeyCode::ArrowUp) {
                        println!(
                            "Rotate model upward by {}",
                            operation_settings.model_rotate_sensitivity
                        );
                        transform.rotate_local_x(-operation_settings.model_rotate_sensitivity);
                    } else if keys.pressed(KeyCode::ArrowDown) {
                        println!(
                            "Rotate model downward by {}",
                            operation_settings.model_rotate_sensitivity
                        );
                        transform.rotate_local_x(operation_settings.model_rotate_sensitivity);
                    } else if keys.pressed(KeyCode::ArrowRight) {
                        println!(
                            "Rotate model to the right by {}",
                            operation_settings.model_rotate_sensitivity
                        );
                        transform.rotate_local_y(operation_settings.model_rotate_sensitivity);
                    } else if keys.pressed(KeyCode::ArrowLeft) {
                        println!(
                            "Rotate model to the left by {}",
                            operation_settings.model_rotate_sensitivity
                        );
                        transform.rotate_local_y(-operation_settings.model_rotate_sensitivity);
                    }

                    // move model
                    if keys.pressed(KeyCode::KeyW) {
                        println!(
                            "Moved model upward by {}",
                            operation_settings.model_reposition_sensitivity
                        );
                        model.y += operation_settings.model_reposition_sensitivity;
                        transform.translation.y = model.y;
                    } else if keys.pressed(KeyCode::KeyS) {
                        println!(
                            "Moved model downward by {}",
                            operation_settings.model_reposition_sensitivity
                        );
                        model.y -= operation_settings.model_reposition_sensitivity;
                        transform.translation.y = model.y;
                    } else if keys.pressed(KeyCode::KeyD) {
                        println!(
                            "Moved model to the right by {}",
                            operation_settings.model_reposition_sensitivity
                        );
                        model.x += operation_settings.model_reposition_sensitivity;
                        transform.translation.x = model.x;
                    } else if keys.pressed(KeyCode::KeyA) {
                        println!(
                            "Moved model to the left by {}",
                            operation_settings.model_reposition_sensitivity
                        );
                        model.x -= operation_settings.model_reposition_sensitivity;
                        transform.translation.x = model.x;
                    }
                }
            }
        }
        Err(_) => {
            return;
        }
    }
}
