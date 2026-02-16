use crate::app::state::GridState;
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseType {
    Fbas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LayerCode {
    Undr00,
    Body01,
    Sock02,
    Fot103,
    Lwr104,
    Shrt05,
    Lwr206,
    Fot207,
    Lwr308,
    Hand09,
    Outr10,
    Neck11,
    Face12,
    Hair13,
    Head14,
    Over15,
}

impl LayerCode {
    pub const ALL: [LayerCode; 16] = [
        LayerCode::Undr00,
        LayerCode::Body01,
        LayerCode::Sock02,
        LayerCode::Fot103,
        LayerCode::Lwr104,
        LayerCode::Shrt05,
        LayerCode::Lwr206,
        LayerCode::Fot207,
        LayerCode::Lwr308,
        LayerCode::Hand09,
        LayerCode::Outr10,
        LayerCode::Neck11,
        LayerCode::Face12,
        LayerCode::Hair13,
        LayerCode::Head14,
        LayerCode::Over15,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            LayerCode::Undr00 => "00undr",
            LayerCode::Body01 => "01body",
            LayerCode::Sock02 => "02sock",
            LayerCode::Fot103 => "03fot1",
            LayerCode::Lwr104 => "04lwr1",
            LayerCode::Shrt05 => "05shrt",
            LayerCode::Lwr206 => "06lwr2",
            LayerCode::Fot207 => "07fot2",
            LayerCode::Lwr308 => "08lwr3",
            LayerCode::Hand09 => "09hand",
            LayerCode::Outr10 => "10outr",
            LayerCode::Neck11 => "11neck",
            LayerCode::Face12 => "12face",
            LayerCode::Hair13 => "13hair",
            LayerCode::Head14 => "14head",
            LayerCode::Over15 => "15over",
        }
    }

    pub fn from_code(raw: &str) -> Option<Self> {
        Some(match raw {
            "00undr" => LayerCode::Undr00,
            "01body" => LayerCode::Body01,
            "02sock" => LayerCode::Sock02,
            "03fot1" => LayerCode::Fot103,
            "04lwr1" => LayerCode::Lwr104,
            "05shrt" => LayerCode::Shrt05,
            "06lwr2" => LayerCode::Lwr206,
            "07fot2" => LayerCode::Fot207,
            "08lwr3" => LayerCode::Lwr308,
            "09hand" => LayerCode::Hand09,
            "10outr" => LayerCode::Outr10,
            "11neck" => LayerCode::Neck11,
            "12face" => LayerCode::Face12,
            "13hair" => LayerCode::Hair13,
            "14head" => LayerCode::Head14,
            "15over" => LayerCode::Over15,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Lower,
    Footwear,
    Head,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Special {
    E,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutfitSetKey {
    pub name: String,
    pub version: u8,
    pub palette: Option<char>,
}

#[derive(Debug, Clone)]
pub struct PartId {
    pub base: BaseType,
    pub layer: LayerCode,
    pub name: String,
    pub version: u8,
    pub palette: Option<char>,
    pub special: Option<Special>,
}

#[derive(Debug, Clone)]
pub struct PartDef {
    pub part_key: String,
    pub part_id: PartId,
    pub slot: Option<Slot>,
    pub image_path: String,
}

impl PartDef {
    pub fn set_key(&self) -> OutfitSetKey {
        OutfitSetKey {
            name: self.part_id.name.clone(),
            version: self.part_id.version,
            palette: self.part_id.palette,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PartCatalog {
    pub parts: Vec<PartDef>,
    pub by_key: HashMap<String, usize>,
    pub by_layer: BTreeMap<LayerCode, Vec<usize>>,
    pub sets: HashMap<OutfitSetKey, Vec<usize>>,
    pub paired_required_sets: HashSet<OutfitSetKey>,
}

impl PartCatalog {
    pub fn index_by_key(&self, key: &str) -> Option<usize> {
        self.by_key.get(key).copied()
    }

    pub fn layer_indices(&self, layer: LayerCode) -> &[usize] {
        self.by_layer
            .get(&layer)
            .map_or(&[] as &[usize], Vec::as_slice)
    }
}

#[derive(Debug, Clone)]
pub struct PaletteDef {
    pub palette_key: String,
    pub image_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct PaletteCatalog {
    pub palettes: Vec<PaletteDef>,
    pub by_key: HashMap<String, usize>,
}

#[derive(Debug, Clone, Default)]
pub struct EquippedParts {
    pub by_layer: BTreeMap<LayerCode, usize>,
}

impl EquippedParts {
    pub fn set_defaults(&mut self, catalog: &PartCatalog) {
        self.by_layer.clear();
        for layer in [
            LayerCode::Body01,
            LayerCode::Sock02,
            LayerCode::Fot103,
            LayerCode::Lwr104,
            LayerCode::Shrt05,
            LayerCode::Hair13,
        ] {
            if let Some(index) = catalog.by_layer.get(&layer).and_then(|parts| parts.first()) {
                self.equip_by_index(catalog, *index);
            }
        }
    }

    pub fn equip_by_index(&mut self, catalog: &PartCatalog, index: usize) {
        let Some(part) = catalog.parts.get(index) else {
            return;
        };
        if let Some(slot) = part.slot {
            self.by_layer.retain(|_, equipped_index| {
                catalog
                    .parts
                    .get(*equipped_index)
                    .is_none_or(|equipped| equipped.slot != Some(slot))
            });
        }
        self.by_layer.insert(part.part_id.layer, index);

        let set_key = part.set_key();
        if catalog.paired_required_sets.contains(&set_key)
            && let Some(paired_indices) = catalog.sets.get(&set_key)
        {
            for paired_index in paired_indices {
                if let Some(paired_part) = catalog.parts.get(*paired_index) {
                    self.by_layer
                        .insert(paired_part.part_id.layer, *paired_index);
                }
            }
        }
    }

    pub fn apply_equipped_keys(&mut self, catalog: &PartCatalog, keys: &[String]) {
        self.by_layer.clear();
        for key in keys {
            if let Some(index) = catalog.index_by_key(key) {
                self.equip_by_index(catalog, index);
            }
        }
    }

    pub fn equipped_part_keys(&self, catalog: &PartCatalog) -> Vec<String> {
        let mut keys = Vec::new();
        for layer in LayerCode::ALL {
            let Some(index) = self.by_layer.get(&layer) else {
                continue;
            };
            let Some(part) = catalog.parts.get(*index) else {
                continue;
            };
            keys.push(part.part_key.clone());
        }
        keys
    }

    pub fn visible_layer_map(&self, catalog: &PartCatalog) -> BTreeMap<LayerCode, usize> {
        let mut map = self.by_layer.clone();
        let hat = map
            .get(&LayerCode::Head14)
            .and_then(|index| catalog.parts.get(*index));
        let hair = map
            .get(&LayerCode::Hair13)
            .and_then(|index| catalog.parts.get(*index));

        let has_hat = hat.is_some();
        let hat_requires_no_hair = hat
            .map(|part| part.part_id.special == Some(Special::E))
            .unwrap_or(false);
        let hair_requires_no_hat = hair
            .map(|part| part.part_id.special == Some(Special::E))
            .unwrap_or(false);

        if hat_requires_no_hair || (has_hat && hair_requires_no_hat) {
            map.remove(&LayerCode::Hair13);
        }
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AtlasSignature {
    columns: u32,
    cell_w: u32,
    cell_h: u32,
    offset_x: u32,
    offset_y: u32,
}

impl AtlasSignature {
    fn from_grid(grid: &GridState) -> Self {
        Self {
            columns: grid.columns.max(1),
            cell_w: grid.cell_width.max(1),
            cell_h: grid.cell_height.max(1),
            offset_x: grid.offset_x,
            offset_y: grid.offset_y,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct PaperDollState {
    pub catalog: PartCatalog,
    pub palette_catalog: PaletteCatalog,
    pub equipped: EquippedParts,
    pub pending_equipped_keys: Option<Vec<String>>,
    pub image_handles: HashMap<String, Handle<Image>>,
    pub palette_image_handles: HashMap<String, Handle<Image>>,
    pub atlas_layouts: HashMap<String, Handle<TextureAtlasLayout>>,
    pub loaded: bool,
    pub load_errors: Vec<String>,
    atlas_signature: Option<AtlasSignature>,
}

impl PaperDollState {
    pub fn visible_layer_map(&self) -> BTreeMap<LayerCode, usize> {
        self.equipped.visible_layer_map(&self.catalog)
    }
}

pub struct LayersFeaturePlugin;

impl Plugin for LayersFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaperDollState>().add_systems(
            Update,
            (
                load_paper_doll_catalog_once,
                apply_pending_equipped_keys,
                sync_paper_doll_atlas_layouts,
            ),
        );
    }
}

fn load_paper_doll_catalog_once(asset_server: Res<AssetServer>, mut state: ResMut<PaperDollState>) {
    if state.loaded {
        return;
    }
    let (catalog, errors) = scan_part_catalog();
    let palette_catalog = scan_palette_catalog();
    state.image_handles.clear();
    state.palette_image_handles.clear();
    for part in &catalog.parts {
        state.image_handles.insert(
            part.part_key.clone(),
            asset_server.load(part.image_path.clone()),
        );
    }
    for palette in &palette_catalog.palettes {
        state.palette_image_handles.insert(
            palette.palette_key.clone(),
            asset_server.load(palette.image_path.clone()),
        );
    }
    if let Some(keys) = state.pending_equipped_keys.take() {
        if keys.is_empty() {
            state.equipped.set_defaults(&catalog);
        } else {
            state.equipped.apply_equipped_keys(&catalog, &keys);
        }
    } else {
        state.equipped.set_defaults(&catalog);
    }
    state.catalog = catalog;
    state.palette_catalog = palette_catalog;
    state.load_errors = errors;
    state.loaded = true;
}

fn apply_pending_equipped_keys(mut state: ResMut<PaperDollState>) {
    if !state.loaded {
        return;
    }
    let Some(keys) = state.pending_equipped_keys.take() else {
        return;
    };
    let PaperDollState {
        catalog, equipped, ..
    } = &mut *state;
    if keys.is_empty() {
        equipped.set_defaults(catalog);
    } else {
        equipped.apply_equipped_keys(catalog, &keys);
    }
}

fn sync_paper_doll_atlas_layouts(
    images: Res<Assets<Image>>,
    grid: Res<GridState>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut state: ResMut<PaperDollState>,
) {
    if !state.loaded {
        return;
    }

    let signature = AtlasSignature::from_grid(&grid);
    if state.atlas_signature != Some(signature) {
        state.atlas_layouts.clear();
        state.atlas_signature = Some(signature);
    }

    let keys: Vec<String> = state.image_handles.keys().cloned().collect();
    for part_key in keys {
        if state.atlas_layouts.contains_key(&part_key) {
            continue;
        }
        let Some(image_handle) = state.image_handles.get(&part_key) else {
            continue;
        };
        let Some(image) = images.get(image_handle) else {
            continue;
        };
        let rows = derive_rows_from_image(Some(image.size()), &grid);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(grid.cell_width.max(1), grid.cell_height.max(1)),
            grid.columns.max(1),
            rows,
            Some(UVec2::ZERO),
            Some(UVec2::new(grid.offset_x, grid.offset_y)),
        );
        let handle = atlas_layouts.add(layout);
        state.atlas_layouts.insert(part_key, handle);
    }
}

pub fn parse_part_filename(file_name: &str) -> Result<PartId, String> {
    let path = Path::new(file_name);
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return Err("Missing extension".to_string());
    };
    if !ext.eq_ignore_ascii_case("png") {
        return Err("Not a png file".to_string());
    }

    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Err("Missing file stem".to_string());
    };
    let parts: Vec<&str> = stem.split('_').collect();
    if parts.len() < 4 {
        return Err("Expected at least 4 filename segments".to_string());
    }
    if parts[0] != "fbas" {
        return Err("Base prefix must be fbas".to_string());
    }

    let Some(layer) = LayerCode::from_code(parts[1]) else {
        return Err(format!("Unknown layer code: {}", parts[1]));
    };

    let mut end = parts.len();
    let special = if parts[end - 1] == "e" {
        end -= 1;
        Some(Special::E)
    } else {
        None
    };
    if end < 4 {
        return Err("Missing name/version segments".to_string());
    }
    let version_palette = parts[end - 1];
    let name = parts[2..end - 1].join("_");
    if name.is_empty() {
        return Err("Part name is empty".to_string());
    }

    let chars: Vec<char> = version_palette.chars().collect();
    if chars.len() < 2 || !chars[0].is_ascii_digit() || !chars[1].is_ascii_digit() {
        return Err("Version token must start with two digits".to_string());
    }
    if chars.len() > 3 {
        return Err("Version token must be 2 digits, optional palette suffix".to_string());
    }
    let version = version_palette[0..2]
        .parse::<u8>()
        .map_err(|_| "Invalid version digits".to_string())?;
    let palette = if chars.len() == 3 {
        let palette = chars[2].to_ascii_lowercase();
        if !matches!(palette, 'a' | 'b' | 'c' | 'd' | 'f') {
            return Err(format!("Invalid palette suffix: {palette}"));
        }
        if version != 0 {
            return Err("Palette suffix is only valid on version 00".to_string());
        }
        Some(palette)
    } else {
        None
    };

    Ok(PartId {
        base: BaseType::Fbas,
        layer,
        name,
        version,
        palette,
        special,
    })
}

fn scan_part_catalog() -> (PartCatalog, Vec<String>) {
    let mut errors = Vec::new();
    let mut parts = Vec::new();
    let mut seen_asset_paths = HashSet::new();

    for root in candidate_assets_roots() {
        let Ok(canonical_root) = std::fs::canonicalize(&root) else {
            continue;
        };
        let mut stack = vec![canonical_root.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if !file_name.starts_with("fbas_") {
                    continue;
                }

                match parse_part_filename(file_name) {
                    Ok(part_id) => {
                        let Ok(canonical_path) = std::fs::canonicalize(&path) else {
                            continue;
                        };
                        let Ok(relative_path) = canonical_path.strip_prefix(&canonical_root) else {
                            continue;
                        };
                        let asset_path = relative_path.to_string_lossy().replace('\\', "/");
                        if !seen_asset_paths.insert(asset_path.clone()) {
                            continue;
                        }

                        let mut version_token = format!("{:02}", part_id.version);
                        if let Some(palette) = part_id.palette {
                            version_token.push(palette);
                        }
                        let mut part_key = format!(
                            "{}/{}/{}",
                            part_id.layer.as_str(),
                            part_id.name,
                            version_token
                        );
                        if part_id.special == Some(Special::E) {
                            part_key.push_str("/e");
                        }

                        parts.push(PartDef {
                            part_key,
                            slot: slot_for_layer(part_id.layer),
                            image_path: asset_path,
                            part_id,
                        });
                    }
                    Err(err) => errors.push(format!("{}: {err}", path.display())),
                }
            }
        }
    }

    (build_catalog(parts), errors)
}

fn scan_palette_catalog() -> PaletteCatalog {
    let mut palettes = Vec::new();
    let mut seen_asset_paths = HashSet::new();

    for root in candidate_assets_roots() {
        let Ok(canonical_root) = std::fs::canonicalize(&root) else {
            continue;
        };
        let mut stack = vec![canonical_root.clone()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                    continue;
                };
                if !ext.eq_ignore_ascii_case("png") {
                    continue;
                }
                let Ok(canonical_path) = std::fs::canonicalize(&path) else {
                    continue;
                };
                let Ok(relative_path) = canonical_path.strip_prefix(&canonical_root) else {
                    continue;
                };
                let asset_path = relative_path.to_string_lossy().replace('\\', "/");
                if !is_palette_asset_path(&asset_path) {
                    continue;
                }
                if !seen_asset_paths.insert(asset_path.clone()) {
                    continue;
                }
                palettes.push(PaletteDef {
                    palette_key: palette_key_from_asset_path(&asset_path),
                    image_path: asset_path,
                });
            }
        }
    }

    build_palette_catalog(palettes)
}

fn build_palette_catalog(mut palettes: Vec<PaletteDef>) -> PaletteCatalog {
    palettes.sort_by(|a, b| a.palette_key.cmp(&b.palette_key));
    let mut catalog = PaletteCatalog {
        palettes,
        ..Default::default()
    };
    for (index, palette) in catalog.palettes.iter().enumerate() {
        catalog.by_key.insert(palette.palette_key.clone(), index);
    }
    catalog
}

fn is_palette_asset_path(asset_path: &str) -> bool {
    let lower = asset_path.to_ascii_lowercase();
    lower.starts_with("palettes/") || lower.contains("/palettes/")
}

fn palette_key_from_asset_path(asset_path: &str) -> String {
    let lower = asset_path.to_ascii_lowercase();
    if lower.ends_with(".png") {
        return asset_path[0..asset_path.len() - 4].to_string();
    }
    asset_path.to_string()
}

fn build_catalog(parts: Vec<PartDef>) -> PartCatalog {
    let mut catalog = PartCatalog {
        parts,
        ..Default::default()
    };
    for (index, part) in catalog.parts.iter().enumerate() {
        catalog.by_key.insert(part.part_key.clone(), index);
        catalog
            .by_layer
            .entry(part.part_id.layer)
            .or_default()
            .push(index);
        catalog.sets.entry(part.set_key()).or_default().push(index);
    }
    for indices in catalog.by_layer.values_mut() {
        indices.sort_by_key(|index| {
            let part = &catalog.parts[*index];
            let base_key = match part.part_id.base {
                BaseType::Fbas => 0u8,
            };
            (
                base_key,
                part.part_id.name.as_str(),
                part.part_id.version,
                part.part_id.palette,
                part.part_id.special,
            )
        });
    }
    for (set_key, indices) in &catalog.sets {
        let layers: HashSet<LayerCode> = indices
            .iter()
            .filter_map(|index| catalog.parts.get(*index).map(|part| part.part_id.layer))
            .collect();
        let has_cloak_pair =
            layers.contains(&LayerCode::Undr00) && layers.contains(&LayerCode::Neck11);
        if has_cloak_pair {
            catalog.paired_required_sets.insert(set_key.clone());
        }
    }
    catalog
}

fn slot_for_layer(layer: LayerCode) -> Option<Slot> {
    Some(match layer {
        LayerCode::Lwr104 | LayerCode::Lwr206 | LayerCode::Lwr308 => Slot::Lower,
        LayerCode::Fot103 | LayerCode::Fot207 => Slot::Footwear,
        LayerCode::Head14 => Slot::Head,
        _ => return None,
    })
}

fn derive_rows_from_image(image_size: Option<UVec2>, grid: &GridState) -> u32 {
    let fallback = grid.rows.max(1);
    let Some(size) = image_size else {
        return fallback;
    };
    let cell_h = grid.cell_height.max(1);
    if size.y <= grid.offset_y {
        return fallback;
    }
    let available = size.y - grid.offset_y;
    (available / cell_h).max(1)
}

fn candidate_assets_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    let Ok(cwd) = std::env::current_dir() else {
        return roots;
    };
    roots.push(cwd.join("assets"));
    roots.push(cwd.join("asset_hander").join("assets"));
    for ancestor in cwd.ancestors() {
        roots.push(ancestor.join("assets"));
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_filename_with_palette_and_special() {
        let part = parse_part_filename("fbas_14head_headscarf_00b_e.png").expect("parse");
        assert_eq!(part.layer, LayerCode::Head14);
        assert_eq!(part.name, "headscarf");
        assert_eq!(part.version, 0);
        assert_eq!(part.palette, Some('b'));
        assert_eq!(part.special, Some(Special::E));
    }

    #[test]
    fn parse_filename_rejects_non_zero_palette_version() {
        let err = parse_part_filename("fbas_14head_hat_01a.png").expect_err("must fail");
        assert!(err.contains("Palette"));
    }

    #[test]
    fn palette_asset_path_is_detected_in_palettes_folder() {
        assert!(is_palette_asset_path(
            "images/Mana Seed Sprite System v1.4a/palettes/mana seed skin ramps.png"
        ));
        assert!(is_palette_asset_path(
            "palettes/base ramps/skin color base ramps.png"
        ));
        assert!(!is_palette_asset_path(
            "images/Mana Seed Sprite System v1.4a/farmer_base_sheets/fbas_01body_human_00.png"
        ));
    }

    #[test]
    fn palette_key_strips_png_extension() {
        let key = palette_key_from_asset_path(
            "images/Mana Seed Sprite System v1.4a/palettes/base ramps/skin color base ramps.png",
        );
        assert_eq!(
            key,
            "images/Mana Seed Sprite System v1.4a/palettes/base ramps/skin color base ramps"
        );
    }

    #[test]
    fn compose_hides_hair_when_hat_has_e() {
        let hair = PartDef {
            part_key: "13hair/mohawk/00/e".to_string(),
            part_id: PartId {
                base: BaseType::Fbas,
                layer: LayerCode::Hair13,
                name: "mohawk".to_string(),
                version: 0,
                palette: None,
                special: Some(Special::E),
            },
            slot: None,
            image_path: "images/test/hair.png".to_string(),
        };
        let hat = PartDef {
            part_key: "14head/headscarf/00b/e".to_string(),
            part_id: PartId {
                base: BaseType::Fbas,
                layer: LayerCode::Head14,
                name: "headscarf".to_string(),
                version: 0,
                palette: Some('b'),
                special: Some(Special::E),
            },
            slot: Some(Slot::Head),
            image_path: "images/test/hat.png".to_string(),
        };

        let mut catalog = PartCatalog::default();
        catalog.parts = vec![hair, hat];
        catalog.by_key.insert(catalog.parts[0].part_key.clone(), 0);
        catalog.by_key.insert(catalog.parts[1].part_key.clone(), 1);

        let mut equipped = EquippedParts::default();
        equipped.by_layer.insert(LayerCode::Hair13, 0);
        equipped.by_layer.insert(LayerCode::Head14, 1);

        let visible = equipped.visible_layer_map(&catalog);
        assert!(!visible.contains_key(&LayerCode::Hair13));
        assert!(visible.contains_key(&LayerCode::Head14));
    }
}
