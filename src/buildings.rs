use bevy::prelude::*;
use rand::Rng;

pub fn spawn_random_building(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
)
{
    let mut rng = rand::thread_rng();
    let height = rng.gen_range(1.0..=10.0);
    let color = Color::srgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());
    let x = rng.gen_range(-240.0..=240.0);
    let z = rng.gen_range(-240.0..=240.0);
    let y = height / 2.0;
    let half_size = Vec3::new(1.0, height / 2.0, 1.0);
    let transform = Transform::from_xyz(x, y, z);
    let mesh = meshes.add(Cuboid { half_size });
    let material = materials.add(StandardMaterial { base_color: color, ..default() });
    commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), transform));
}