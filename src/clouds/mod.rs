#![doc = include_str!("README.md")]

mod compute;
/// Controls the compute shader which renders the volumetric clouds.
pub mod config;
mod images;
mod render;
mod skybox;
mod uniforms;
use bevy::prelude::*;

use crate::clouds::{
    compute::CameraMatrices,
    config::CloudsConfig,
    images::build_images,
    render::{CloudsMaterial, CloudsShaderPlugin},
    skybox::{SkyboxMaterials, init_skybox_mesh, setup_daylight, update_skybox_transform},
    uniforms::CloudsImage,
};

use self::compute::CloudsComputePlugin;

/// A plugin for rendering clouds.
///
/// The configuration of the clouds can be changed using the [`CloudsConfig`] resource.
pub struct CloudsPlugin;

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CloudsConfig::default())
            .add_plugins((CloudsComputePlugin, CloudsShaderPlugin))
            .add_systems(Startup, (clouds_setup, setup_daylight))
            .add_systems(
                PostUpdate,
                (update_skybox_transform, update_camera_matrices)
                    .after(TransformSystems::Propagate),
            );
    }
}

fn clouds_setup(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudsMaterial>>,
) {
    let (cloud_render_image, cloud_atlas_image, cloud_worley_image) = build_images(images);

    let material = materials.add(CloudsMaterial {
        cloud_render_image: cloud_render_image.clone(),
        cloud_atlas_image: cloud_atlas_image.clone(),
        cloud_worley_image: cloud_worley_image.clone(),
    });
    init_skybox_mesh(
        &mut commands,
        meshes,
        SkyboxMaterials::from_one_material(MeshMaterial3d(material.clone())),
    );
    commands.insert_resource(CloudsImage {
        cloud_render_image,
        cloud_atlas_image,
        cloud_worley_image,
    });
    commands.insert_resource(CameraMatrices {
        translation: Vec3::ZERO,
        inverse_camera_projection: Mat4::IDENTITY,
        inverse_camera_view: Mat4::IDENTITY,
    });
}

fn update_camera_matrices(
    cam_query: Single<(&GlobalTransform, &Camera)>,
    mut config: ResMut<CameraMatrices>,
) {
    let (camera_transform, camera) = *cam_query;
    config.translation = camera_transform.translation();
    config.inverse_camera_view = camera_transform.to_matrix();
    config.inverse_camera_projection = camera.computed.clip_from_view.inverse();
}
