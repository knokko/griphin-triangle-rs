use cgmath::Vector2;
use griphin::*;

use std::sync::Arc;

pub fn user_start(instance: Arc<dyn Instance>) {
    println!("Start the user part");
    let vertex_buffer = create_vertex_buffer(instance.get_gateway());
    let shader_pair = create_shader_pair(instance.get_shader_manager());
    let (abstract_grid_group, color_grid_id, depth_stencil_grid_id)
        = create_abstract_grid_group(instance.as_ref());
    let graphics_pipeline =
        abstract_grid_group.create_graphics_pipeline(&shader_pair, PrimitiveTopology::Triangles);

    // TODO Don't hardcode width and height
    let (width, height) = (800, 500);
    let grid_group = abstract_grid_group.create_concrete(width, height);
    let render_flow = create_render_flow(
        abstract_grid_group.as_ref(),
        &graphics_pipeline,
        color_grid_id,
        depth_stencil_grid_id
    );
}

fn create_render_flow(
    abstract_grid_group: &dyn AbstractGridGroup,
    graphics_pipeline: &Arc<dyn GraphicsPipeline>,
    abstract_color_grid_id: AbstractGridID,
    abstract_depth_stencil_grid_id: AbstractGridID
) -> Arc<dyn RenderFlow> {
    let mut builder = abstract_grid_group.create_render_flow_builder();
    let color_grid_id = builder.add_grid_node(abstract_color_grid_id);
    let depth_stencil_grid_id = builder.add_grid_node(abstract_depth_stencil_grid_id);
    let drawing_node_builder = DrawingNodeBuilder {
        pipeline: Arc::clone(graphics_pipeline),
        // TODO What about the implicit inputs and outputs? Like the depth-stencil grid
        inputs: vec![],
        outputs: vec![
            DrawingNodeOutput {
                destination: DrawingNodeOutputDestination::External(abstract_color_grid_id),
                shader_variable_name: str_ref("outputColor")
            }
        ]
    };
    builder.add_drawing_node(drawing_node_builder);

    abstract_grid_group.create_render_flow(builder)
}

fn create_abstract_grid_group(instance: &dyn Instance) -> (Arc<dyn AbstractGridGroup>, AbstractGridID, AbstractGridID) {
    let color_grid = AbstractColorGridBuilder {
        start_operation: ColorStartOperation::Clear,
        purpose: ColorPurpose::Display
    };
    // TODO Do we really need a depth-stencil grid for 2d rendering?
    let depth_stencil_grid = AbstractDepthStencilGridBuilder {
        start_operation: DepthStencilStartOperation::Clear,
        purpose: DepthStencilPurpose::Nothing
    };
    let builder = AbstractGridGroupBuilder {
        color_grids: vec![color_grid],
        depth_stencil_grids: vec![depth_stencil_grid]
    };

    let (grid_group, ids) = instance.create_abstract_grid_group(&builder);
    let color_grid_id = ids.colors[0];
    let depth_stencil_grid_id = ids.depth_stencils[0];
    (grid_group, color_grid_id, depth_stencil_grid_id)
}

fn create_shader_pair(shader_manager: Arc<dyn ShaderManager>) -> ShaderPair {
    let vertex_shader = create_vertex_shader(shader_manager.as_ref());
    let fragment_shader = create_fragment_shader(shader_manager.as_ref());
    ShaderPair::link_by_attribute_names(&vertex_shader, &fragment_shader).expect("No shader problems")
}

fn create_vertex_shader(shader_manager: &dyn ShaderManager) -> Arc<dyn VertexShader> {
    let main_method_content = "
        passPosition = position;
        gl_Position = vec4(position, 0.0, 1.0);
    ";

    let shader_variables = vec![
        VertexShaderVariable::new(
            &str_ref("position"),
            DataType::new(FLOAT, VEC2),
            VertexShaderVariableType::VertexInput
        ),
        VertexShaderVariable::new(
            &str_ref("passPosition"),
            DataType::new(FLOAT, VEC2),
            VertexShaderVariableType::SmoothFragmentOutput
        )
    ];

    shader_manager.create_vertex_shader(
        &str_ref("TheVertexShader"),
        &str_ref(main_method_content),
        &str_ref(""),
        shader_variables,
        Vec::new()
    )
}

fn create_fragment_shader(shader_manager: &dyn ShaderManager) -> Arc<dyn FragmentShader> {
    let main_method_content = "
        outputColor = vec4(passPosition.x, passPosition.y, 1.0, 1.0);
    ";

    let shader_variables = vec![
        FragmentShaderVariable::new(
            &str_ref("passPosition"),
            DataType::new(FLOAT, VEC2),
            FragmentShaderVariableType::SmoothVertexInput
        ),
        FragmentShaderVariable::new(
            &str_ref("outputColor"),
            DataType::new(FLOAT, VEC4),
            FragmentShaderVariableType::ColorOutput
        )
    ];

    shader_manager.create_fragment_shader(
        &str_ref("TheFragmentShader"),
        &str_ref(main_method_content),
        &str_ref(""),
        shader_variables,
        Vec::new()
    )
}

struct SimpleVertexDescription {
    raw: RawVertexDescription,
    position: VertexAttributeHandle
}

impl SimpleVertexDescription {
    fn new() -> Self {
        let mut raw = RawVertexDescription::new();
        let position = raw.add_attribute(
            &str_ref("position"),
            DataType::new(FLOAT, VEC2),
            AttributeKind::Position { max: 1.0 }
        );
        Self { raw, position }
    }
}

impl VertexDescription for SimpleVertexDescription {
    fn get_raw_description(&self) -> &RawVertexDescription {
        &self.raw
    }
}

struct SimpleVertex {
    position: Vector2<f32>
}

impl Vertex<SimpleVertexDescription> for SimpleVertex {
    fn store(&self, store: &mut VertexStoreBuilder, description: &SimpleVertexDescription) {
        store.put_vec2f(description.position, self.position);
    }
}

fn create_vertex_buffer(gateway: Arc<dyn Gateway>) -> Arc<dyn VertexBuffer> {
    let vertices = [
        SimpleVertex { position: Vector2 { x: -0.3, y: -0.3 } },
        SimpleVertex { position: Vector2 { x: 0.3, y: -0.3 } },
        SimpleVertex { position: Vector2 { x: 0.0, y: 0.5 } }
    ];
    let store = VertexStore::new(
        &SimpleVertexDescription::new(), &vertices,
        DebugLevel::All, None
    );
    let usage = VertexBufferUsage::NoIndices { topology: PrimitiveTopology::Triangles };
    gateway.transfer_vertices(&store, usage)
}