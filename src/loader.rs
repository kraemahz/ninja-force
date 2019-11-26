use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    prelude::World,
    renderer::{
        formats::texture::ImageFormat,
        sprite::{SpriteSheet, SpriteSheetFormat},
        Texture,
    },
};

pub fn load_sprite_sheet(
    world: &mut World,
    sprite: &str,
    sprite_sheet: &str,
) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(sprite, ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    loader.load(
        sprite_sheet,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}
