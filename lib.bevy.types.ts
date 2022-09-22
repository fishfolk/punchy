// generated with https://github.com/jakobhellermann/bevy_reflect_ts_type_export
type bool = boolean;
type f32 = number;
type f64 = number;
type i8 = number;
type i16 = number;
type i32 = number;
type i64 = number;
type isize = number;
type u8 = number;
type u16 = number;
type u32 = number;
type u64 = number;
type usize = number;
type Cowstr = String;
type Vec3A = Vec3;
type AnimationPlayer = {
    paused: bool,
    repeat: bool,
    speed: f32,
    elapsed: f32,
    animation_clip: HandleAnimationClip,
};
const AnimationPlayer: BevyType<AnimationPlayer> = { typeName: "bevy_animation::AnimationPlayer" };

type HandleAnimationClip = {
    id: HandleId,
};
const HandleAnimationClip: BevyType<HandleAnimationClip> = { typeName: "bevy_asset::handle::Handle<bevy_animation::AnimationClip>" };

type HandleAudioSink = {
    id: HandleId,
};
const HandleAudioSink: BevyType<HandleAudioSink> = { typeName: "bevy_asset::handle::Handle<bevy_audio::audio_output::AudioSink>" };

type HandleAudioSource = {
    id: HandleId,
};
const HandleAudioSource: BevyType<HandleAudioSource> = { typeName: "bevy_asset::handle::Handle<bevy_audio::audio_source::AudioSource>" };

type HandleGltf = {
    id: HandleId,
};
const HandleGltf: BevyType<HandleGltf> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::Gltf>" };

type HandleGltfMesh = {
    id: HandleId,
};
const HandleGltfMesh: BevyType<HandleGltfMesh> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfMesh>" };

type HandleGltfNode = {
    id: HandleId,
};
const HandleGltfNode: BevyType<HandleGltfNode> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfNode>" };

type HandleGltfPrimitive = {
    id: HandleId,
};
const HandleGltfPrimitive: BevyType<HandleGltfPrimitive> = { typeName: "bevy_asset::handle::Handle<bevy_gltf::GltfPrimitive>" };

type HandleStandardMaterial = {
    id: HandleId,
};
const HandleStandardMaterial: BevyType<HandleStandardMaterial> = { typeName: "bevy_asset::handle::Handle<bevy_pbr::pbr_material::StandardMaterial>" };

type HandleMesh = {
    id: HandleId,
};
const HandleMesh: BevyType<HandleMesh> = { typeName: "bevy_asset::handle::Handle<bevy_render::mesh::mesh::Mesh>" };

type HandleSkinnedMeshInverseBindposes = {
    id: HandleId,
};
const HandleSkinnedMeshInverseBindposes: BevyType<HandleSkinnedMeshInverseBindposes> = { typeName: "bevy_asset::handle::Handle<bevy_render::mesh::mesh::skinning::SkinnedMeshInverseBindposes>" };

type HandleShader = {
    id: HandleId,
};
const HandleShader: BevyType<HandleShader> = { typeName: "bevy_asset::handle::Handle<bevy_render::render_resource::shader::Shader>" };

type HandleImage = {
    id: HandleId,
};
const HandleImage: BevyType<HandleImage> = { typeName: "bevy_asset::handle::Handle<bevy_render::texture::image::Image>" };

type HandleDynamicScene = {
    id: HandleId,
};
const HandleDynamicScene: BevyType<HandleDynamicScene> = { typeName: "bevy_asset::handle::Handle<bevy_scene::dynamic_scene::DynamicScene>" };

type HandleScene = {
    id: HandleId,
};
const HandleScene: BevyType<HandleScene> = { typeName: "bevy_asset::handle::Handle<bevy_scene::scene::Scene>" };

type HandleColorMaterial = {
    id: HandleId,
};
const HandleColorMaterial: BevyType<HandleColorMaterial> = { typeName: "bevy_asset::handle::Handle<bevy_sprite::mesh2d::color_material::ColorMaterial>" };

type HandleTextureAtlas = {
    id: HandleId,
};
const HandleTextureAtlas: BevyType<HandleTextureAtlas> = { typeName: "bevy_asset::handle::Handle<bevy_sprite::texture_atlas::TextureAtlas>" };

type HandleFont = {
    id: HandleId,
};
const HandleFont: BevyType<HandleFont> = { typeName: "bevy_asset::handle::Handle<bevy_text::font::Font>" };

type HandleFontAtlasSet = {
    id: HandleId,
};
const HandleFontAtlasSet: BevyType<HandleFontAtlasSet> = { typeName: "bevy_asset::handle::Handle<bevy_text::font_atlas_set::FontAtlasSet>" };

type HandleId = unknown;
const HandleId: BevyType<HandleId> = { typeName: "bevy_asset::handle::HandleId" };

type Name = {
    hash: u64,
    name: Cowstr,
};
const Name: BevyType<Name> = { typeName: "bevy_core::name::Name" };

type ClearColor = unknown;
const ClearColor: BevyType<ClearColor> = { typeName: "bevy_core_pipeline::clear_color::ClearColor" };

type ClearColorConfig = unknown;
const ClearColorConfig: BevyType<ClearColorConfig> = { typeName: "bevy_core_pipeline::clear_color::ClearColorConfig" };

type Camera2d = {
    clear_color: ClearColorConfig,
};
const Camera2d: BevyType<Camera2d> = { typeName: "bevy_core_pipeline::core_2d::camera_2d::Camera2d" };

type Camera3d = {
    clear_color: ClearColorConfig,
    depth_load_op: Camera3dDepthLoadOp,
};
const Camera3d: BevyType<Camera3d> = { typeName: "bevy_core_pipeline::core_3d::camera_3d::Camera3d" };

type Camera3dDepthLoadOp = unknown;
const Camera3dDepthLoadOp: BevyType<Camera3dDepthLoadOp> = { typeName: "bevy_core_pipeline::core_3d::camera_3d::Camera3dDepthLoadOp" };

type GltfExtras = {
    value: string,
};
const GltfExtras: BevyType<GltfExtras> = { typeName: "bevy_gltf::GltfExtras" };

type Children = unknown;
const Children: BevyType<Children> = { typeName: "bevy_hierarchy::components::children::Children" };

type Parent = unknown;
const Parent: BevyType<Parent> = { typeName: "bevy_hierarchy::components::parent::Parent" };

type CubemapVisibleEntities = {
};
const CubemapVisibleEntities: BevyType<CubemapVisibleEntities> = { typeName: "bevy_pbr::bundle::CubemapVisibleEntities" };

type AmbientLight = {
    color: Color,
    brightness: f32,
};
const AmbientLight: BevyType<AmbientLight> = { typeName: "bevy_pbr::light::AmbientLight" };

type DirectionalLight = {
    color: Color,
    illuminance: f32,
    shadows_enabled: bool,
    shadow_projection: OrthographicProjection,
    shadow_depth_bias: f32,
    shadow_normal_bias: f32,
};
const DirectionalLight: BevyType<DirectionalLight> = { typeName: "bevy_pbr::light::DirectionalLight" };

type DirectionalLightShadowMap = {
    size: usize,
};
const DirectionalLightShadowMap: BevyType<DirectionalLightShadowMap> = { typeName: "bevy_pbr::light::DirectionalLightShadowMap" };

type PointLight = {
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    shadows_enabled: bool,
    shadow_depth_bias: f32,
    shadow_normal_bias: f32,
};
const PointLight: BevyType<PointLight> = { typeName: "bevy_pbr::light::PointLight" };

type PointLightShadowMap = {
    size: usize,
};
const PointLightShadowMap: BevyType<PointLightShadowMap> = { typeName: "bevy_pbr::light::PointLightShadowMap" };

type SpotLight = {
    color: Color,
    intensity: f32,
    range: f32,
    radius: f32,
    shadows_enabled: bool,
    shadow_depth_bias: f32,
    shadow_normal_bias: f32,
    outer_angle: f32,
    inner_angle: f32,
};
const SpotLight: BevyType<SpotLight> = { typeName: "bevy_pbr::light::SpotLight" };

type Camera = {
    viewport: Viewport | null,
    priority: isize,
    is_active: bool,
    depth_calculation: DepthCalculation,
};
const Camera: BevyType<Camera> = { typeName: "bevy_render::camera::camera::Camera" };

type CameraRenderGraph = unknown;
const CameraRenderGraph: BevyType<CameraRenderGraph> = { typeName: "bevy_render::camera::camera::CameraRenderGraph" };

type DepthCalculation = unknown;
const DepthCalculation: BevyType<DepthCalculation> = { typeName: "bevy_render::camera::camera::DepthCalculation" };

type Viewport = unknown;
const Viewport: BevyType<Viewport> = { typeName: "bevy_render::camera::camera::Viewport" };

type OrthographicProjection = {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
    window_origin: WindowOrigin,
    scaling_mode: ScalingMode,
    scale: f32,
    depth_calculation: DepthCalculation,
};
const OrthographicProjection: BevyType<OrthographicProjection> = { typeName: "bevy_render::camera::projection::OrthographicProjection" };

type PerspectiveProjection = {
    fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
};
const PerspectiveProjection: BevyType<PerspectiveProjection> = { typeName: "bevy_render::camera::projection::PerspectiveProjection" };

type Projection = unknown;
const Projection: BevyType<Projection> = { typeName: "bevy_render::camera::projection::Projection" };

type ScalingMode = unknown;
const ScalingMode: BevyType<ScalingMode> = { typeName: "bevy_render::camera::projection::ScalingMode" };

type WindowOrigin = unknown;
const WindowOrigin: BevyType<WindowOrigin> = { typeName: "bevy_render::camera::projection::WindowOrigin" };

type Color = unknown;
const Color: BevyType<Color> = { typeName: "bevy_render::color::Color" };

type SkinnedMesh = {
    inverse_bindposes: HandleSkinnedMeshInverseBindposes,
    joints: Entity[],
};
const SkinnedMesh: BevyType<SkinnedMesh> = { typeName: "bevy_render::mesh::mesh::skinning::SkinnedMesh" };

type Aabb = {
    center: Vec3A,
    half_extents: Vec3A,
};
const Aabb: BevyType<Aabb> = { typeName: "bevy_render::primitives::Aabb" };

type CubemapFrusta = {
};
const CubemapFrusta: BevyType<CubemapFrusta> = { typeName: "bevy_render::primitives::CubemapFrusta" };

type Frustum = {
};
const Frustum: BevyType<Frustum> = { typeName: "bevy_render::primitives::Frustum" };

type Msaa = {
    samples: u32,
};
const Msaa: BevyType<Msaa> = { typeName: "bevy_render::view::Msaa" };

type ComputedVisibility = {
    is_visible_in_hierarchy: bool,
    is_visible_in_view: bool,
};
const ComputedVisibility: BevyType<ComputedVisibility> = { typeName: "bevy_render::view::visibility::ComputedVisibility" };

type Visibility = {
    is_visible: bool,
};
const Visibility: BevyType<Visibility> = { typeName: "bevy_render::view::visibility::Visibility" };

type VisibleEntities = {
};
const VisibleEntities: BevyType<VisibleEntities> = { typeName: "bevy_render::view::visibility::VisibleEntities" };

type Mesh2dHandle = unknown;
const Mesh2dHandle: BevyType<Mesh2dHandle> = { typeName: "bevy_sprite::mesh2d::mesh::Mesh2dHandle" };

type Anchor = unknown;
const Anchor: BevyType<Anchor> = { typeName: "bevy_sprite::sprite::Anchor" };

type Sprite = {
    color: Color,
    flip_x: bool,
    flip_y: bool,
    custom_size: Vec2 | null,
    anchor: Anchor,
};
const Sprite: BevyType<Sprite> = { typeName: "bevy_sprite::sprite::Sprite" };

type HorizontalAlign = unknown;
const HorizontalAlign: BevyType<HorizontalAlign> = { typeName: "bevy_text::text::HorizontalAlign" };

type TextAlignment = {
    vertical: VerticalAlign,
    horizontal: HorizontalAlign,
};
const TextAlignment: BevyType<TextAlignment> = { typeName: "bevy_text::text::TextAlignment" };

type TextSection = {
    value: string,
    style: TextStyle,
};
const TextSection: BevyType<TextSection> = { typeName: "bevy_text::text::TextSection" };

type TextStyle = {
    font: HandleFont,
    font_size: f32,
    color: Color,
};
const TextStyle: BevyType<TextStyle> = { typeName: "bevy_text::text::TextStyle" };

type VerticalAlign = unknown;
const VerticalAlign: BevyType<VerticalAlign> = { typeName: "bevy_text::text::VerticalAlign" };

type Stopwatch = {
    elapsed: Duration,
    paused: bool,
};
const Stopwatch: BevyType<Stopwatch> = { typeName: "bevy_time::stopwatch::Stopwatch" };

type Time = {
    delta: Duration,
    last_update: Instant | null,
    delta_seconds_f64: f64,
    delta_seconds: f32,
    seconds_since_startup: f64,
    time_since_startup: Duration,
    startup: Instant,
};
const Time: BevyType<Time> = { typeName: "bevy_time::time::Time" };

type Timer = {
    stopwatch: Stopwatch,
    duration: Duration,
    repeating: bool,
    finished: bool,
    times_finished_this_tick: u32,
};
const Timer: BevyType<Timer> = { typeName: "bevy_time::timer::Timer" };

type GlobalTransform = unknown;
const GlobalTransform: BevyType<GlobalTransform> = { typeName: "bevy_transform::components::global_transform::GlobalTransform" };

type Transform = {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
};
const Transform: BevyType<Transform> = { typeName: "bevy_transform::components::transform::Transform" };

type FocusPolicy = unknown;
const FocusPolicy: BevyType<FocusPolicy> = { typeName: "bevy_ui::focus::FocusPolicy" };

type Interaction = unknown;
const Interaction: BevyType<Interaction> = { typeName: "bevy_ui::focus::Interaction" };

type Size = {
    width: f32,
    height: f32,
};
const Size: BevyType<Size> = { typeName: "bevy_ui::geometry::Size" };

type SizeVal = {
    width: Val,
    height: Val,
};
const SizeVal: BevyType<SizeVal> = { typeName: "bevy_ui::geometry::Size<bevy_ui::ui_node::Val>" };

type UiRectVal = {
    left: Val,
    right: Val,
    top: Val,
    bottom: Val,
};
const UiRectVal: BevyType<UiRectVal> = { typeName: "bevy_ui::geometry::UiRect<bevy_ui::ui_node::Val>" };

type AlignContent = unknown;
const AlignContent: BevyType<AlignContent> = { typeName: "bevy_ui::ui_node::AlignContent" };

type AlignItems = unknown;
const AlignItems: BevyType<AlignItems> = { typeName: "bevy_ui::ui_node::AlignItems" };

type AlignSelf = unknown;
const AlignSelf: BevyType<AlignSelf> = { typeName: "bevy_ui::ui_node::AlignSelf" };

type CalculatedSize = {
    size: Size,
};
const CalculatedSize: BevyType<CalculatedSize> = { typeName: "bevy_ui::ui_node::CalculatedSize" };

type Direction = unknown;
const Direction: BevyType<Direction> = { typeName: "bevy_ui::ui_node::Direction" };

type Display = unknown;
const Display: BevyType<Display> = { typeName: "bevy_ui::ui_node::Display" };

type FlexDirection = unknown;
const FlexDirection: BevyType<FlexDirection> = { typeName: "bevy_ui::ui_node::FlexDirection" };

type FlexWrap = unknown;
const FlexWrap: BevyType<FlexWrap> = { typeName: "bevy_ui::ui_node::FlexWrap" };

type JustifyContent = unknown;
const JustifyContent: BevyType<JustifyContent> = { typeName: "bevy_ui::ui_node::JustifyContent" };

type Overflow = unknown;
const Overflow: BevyType<Overflow> = { typeName: "bevy_ui::ui_node::Overflow" };

type PositionType = unknown;
const PositionType: BevyType<PositionType> = { typeName: "bevy_ui::ui_node::PositionType" };

type Style = {
    display: Display,
    position_type: PositionType,
    direction: Direction,
    flex_direction: FlexDirection,
    flex_wrap: FlexWrap,
    align_items: AlignItems,
    align_self: AlignSelf,
    align_content: AlignContent,
    justify_content: JustifyContent,
    position: UiRectVal,
    margin: UiRectVal,
    padding: UiRectVal,
    border: UiRectVal,
    flex_grow: f32,
    flex_shrink: f32,
    flex_basis: Val,
    size: SizeVal,
    min_size: SizeVal,
    max_size: SizeVal,
    aspect_ratio: f32 | null,
    overflow: Overflow,
};
const Style: BevyType<Style> = { typeName: "bevy_ui::ui_node::Style" };

type UiColor = unknown;
const UiColor: BevyType<UiColor> = { typeName: "bevy_ui::ui_node::UiColor" };

type UiImage = unknown;
const UiImage: BevyType<UiImage> = { typeName: "bevy_ui::ui_node::UiImage" };

type Val = unknown;
const Val: BevyType<Val> = { typeName: "bevy_ui::ui_node::Val" };

type Button = {
};
const Button: BevyType<Button> = { typeName: "bevy_ui::widget::button::Button" };

type ImageMode = unknown;
const ImageMode: BevyType<ImageMode> = { typeName: "bevy_ui::widget::image::ImageMode" };

type Rangef32 = unknown;
const Rangef32: BevyType<Rangef32> = { typeName: "core::ops::range::Range<f32>" };

type OptionString = unknown;
const OptionString: BevyType<OptionString> = { typeName: "core::option::Option<alloc::string::String>" };

type Optionf32 = unknown;
const Optionf32: BevyType<Optionf32> = { typeName: "core::option::Option<f32>" };

type Duration = unknown;
const Duration: BevyType<Duration> = { typeName: "core::time::Duration" };

type Mat3 = {
    x_axis: Vec3,
    y_axis: Vec3,
    z_axis: Vec3,
};
const Mat3: BevyType<Mat3> = { typeName: "glam::f32::mat3::Mat3" };

type Mat2 = {
    x_axis: Vec2,
    y_axis: Vec2,
};
const Mat2: BevyType<Mat2> = { typeName: "glam::f32::sse2::mat2::Mat2" };

type Mat4 = {
    x_axis: Vec4,
    y_axis: Vec4,
    z_axis: Vec4,
    w_axis: Vec4,
};
const Mat4: BevyType<Mat4> = { typeName: "glam::f32::sse2::mat4::Mat4" };

type Quat = unknown;
const Quat: BevyType<Quat> = { typeName: "glam::f32::sse2::quat::Quat" };

type Vec4 = {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
};
const Vec4: BevyType<Vec4> = { typeName: "glam::f32::sse2::vec4::Vec4" };

type Vec2 = {
    x: f32,
    y: f32,
};
const Vec2: BevyType<Vec2> = { typeName: "glam::f32::vec2::Vec2" };

type Vec3 = {
    x: f32,
    y: f32,
    z: f32,

    lerp(other: Vec3, t: f32): Vec3;
};
const Vec3: BevyType<Vec3> = { typeName: "glam::f32::vec3::Vec3" };

type IVec2 = {
    x: i32,
    y: i32,
};
const IVec2: BevyType<IVec2> = { typeName: "glam::i32::ivec2::IVec2" };

type IVec3 = {
    x: i32,
    y: i32,
    z: i32,
};
const IVec3: BevyType<IVec3> = { typeName: "glam::i32::ivec3::IVec3" };

type IVec4 = {
    x: i32,
    y: i32,
    z: i32,
    w: i32,
};
const IVec4: BevyType<IVec4> = { typeName: "glam::i32::ivec4::IVec4" };

type UVec2 = {
    x: u32,
    y: u32,
};
const UVec2: BevyType<UVec2> = { typeName: "glam::u32::uvec2::UVec2" };

type UVec3 = {
    x: u32,
    y: u32,
    z: u32,
};
const UVec3: BevyType<UVec3> = { typeName: "glam::u32::uvec3::UVec3" };

type UVec4 = {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
};
const UVec4: BevyType<UVec4> = { typeName: "glam::u32::uvec4::UVec4" };

type HashSetString = unknown;
const HashSetString: BevyType<HashSetString> = { typeName: "hashbrown::set::HashSet<alloc::string::String>" };

type Instant = unknown;
const Instant: BevyType<Instant> = { typeName: "std::time::Instant" };
