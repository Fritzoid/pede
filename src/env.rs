use bevy::color::palettes::css::LIGHT_GREEN;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use rand::Rng;
use std::f32::consts::PI;

#[derive(Component)]
struct GameCamera;

pub fn spawn_env(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-1.0, -0.7, -1.0), Vec3::X),
    ));
    // Sky
    commands.spawn((
        Mesh3d(meshes.add(Sphere::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true,
            base_color: Color::linear_rgb(0.1, 0.6, 1.0),
            ..default()
        })),
        Transform::default().with_scale(Vec3::splat(-4000.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5000.0, 5000.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(LIGHT_GREEN))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    spawn_trees(meshes, materials, commands);
    spawn_houses(meshes, materials, commands, asset_server);

    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            pitch_lower_limit: Some(PI / 6.0),
            pitch_upper_limit: Some(PI / 4.0),
            zoom_lower_limit: 10.0,
            zoom_upper_limit: Some(500.0),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        GameCamera,
    ));
}

fn spawn_trees(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_TREES: usize = 75;
    let trunk = meshes.add(Cylinder::default());
    let crown = meshes.add(Sphere::default());
    let trunk_mat = materials.add(Color::linear_rgb(0.4, 0.2, 0.2));
    let crown_mat = materials.add(Color::linear_rgb(0.0, 1.0, 0.0));

    for _i in 0..N_TREES {
        let x = rnd_tree_coord();
        let z = rnd_tree_coord();
        commands.spawn((
            Mesh3d(trunk.clone()),
            MeshMaterial3d(trunk_mat.clone()),
            Transform::from_xyz(x, 0.0, z).with_scale(Vec3::new(0.1, 1.0, 0.1)),
        ));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(crown_mat.clone()),
            Transform::from_xyz(x, 1.0, z),
        ));
    }
}

fn rnd_tree_coord() -> f32 {
    let mut rng = rand::rng();
    let random_number: f32 = if rng.random_bool(0.5) {
        rng.random_range(-25.0..=-2.0)
    } else {
        rng.random_range(2.0..=25.0)
    };
    random_number
}

fn spawn_houses(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    let house_body = meshes.add(Cuboid::default());
    let window = meshes.add(Cuboid {
        half_size: Vec3::new(0.21, 0.21, 0.51),
        ..default()
    });
    let house_body_mat = materials.add(Color::linear_rgb(0.5, 0.5, 0.5));
    let window_mat = materials.add(Color::linear_rgb(0.0, 0.0, 0.0));

    const N_HOUSES: usize = 25;
    for _i in 0..N_HOUSES {
        let x = rnd_tree_coord();
        let z = rnd_tree_coord();
        commands.spawn((
            Mesh3d(house_body.clone()),
            MeshMaterial3d(house_body_mat.clone()),
            Transform::from_xyz(x, 0.5, z),
        ));
        commands.spawn((
            Mesh3d(window.clone()),
            MeshMaterial3d(window_mat.clone()),
            Transform::from_xyz(x, 0.6, z),
        ));
    }

    let scene_handle1 = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("models/low_poly_japan_building/low_poly_japan_building.glb"),
    );
    commands.spawn((
        SceneRoot(scene_handle1.clone()),
        Transform::from_xyz(7.0, 0.0, -14.0),
    ));
    commands.spawn((
        SceneRoot(scene_handle1.clone()),
        Transform::from_xyz(-7.0, 0.0, 14.0),
    ));

    let spike_house = meshes.add(Cone { radius: 1.0, height: 20.0 });
    let spike_house_mat = materials.add(Color::linear_rgb(0.3, 0.3, 0.9));

    commands.spawn((
        Mesh3d(spike_house.clone()),
        MeshMaterial3d(spike_house_mat.clone()),
        Transform::from_xyz(-5.0, 0.0, -12.0),
    ));
    commands.spawn((
        Mesh3d(spike_house.clone()),
        MeshMaterial3d(spike_house_mat.clone()),
        Transform::from_xyz(8.0, 0.0, 18.0).with_scale(Vec3::new(1.0, 5.0, 1.0)),
    ));
}
