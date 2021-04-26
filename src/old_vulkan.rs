#![allow(dead_code)]
#![allow(unused_variables)]

use ash::{Device, Entry, Instance, extensions::ext, extensions::khr, version::{DeviceV1_0, EntryV1_0, InstanceV1_0}, vk::{self, SurfaceKHR, Window}};
use std::{ffi:: { CStr, CString}, mem};
use byte_slice_cast::{AsByteSlice, AsSliceOf};

pub(crate) const MAX_FRAMES_IN_FLIGHT:usize = 2;

#[derive(Clone, Debug)]
struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn find_queue_families(
        instance: &Instance,
        device: vk::PhysicalDevice,
        entry: &Entry,
        surface_khr: vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };
        let surface = khr::Surface::new(entry, instance);

        for (i, queue) in unsafe { instance
            .get_physical_device_queue_family_properties(device) }
            .iter()
            .enumerate()
        {
            let i = i as u32;
            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i);
            }
            if unsafe { surface
                .get_physical_device_surface_support(device, i, surface_khr) }
                .unwrap()
            {
                indices.present_family = Some(i);
            }
            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }

    fn are_same(&self) -> bool {
        self.is_complete() && self.graphics_family == self.present_family
    }
}

struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    surface_formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>
}

impl SwapChainSupportDetails {
    fn query_swap_chain_support(instance: &Instance, entry: &Entry, device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> SwapChainSupportDetails {
        let surface_ext = khr::Surface::new(entry, instance);
        let capabilities = unsafe { surface_ext.get_physical_device_surface_capabilities(device, surface).expect("unable to get capabilities") };
        let surface_formats = unsafe { surface_ext.get_physical_device_surface_formats(device, surface).expect("unable to get surface formats") };
        let present_modes = unsafe { surface_ext.get_physical_device_surface_present_modes(device, surface).expect("unable to get present modes") };

        SwapChainSupportDetails {
            capabilities,
            surface_formats,
            present_modes
        }
    }
}

struct HelloTriangleApplication {
    _entry: Entry,
    instance: Instance,
    debug_utils: Option<ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    _physical_device: vk::PhysicalDevice,
    device: Device,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,
    swap_chain_ext: khr::Swapchain,
    swap_chain: vk::SwapchainKHR,
    _swap_chain_images: Vec<vk::Image>,
    _swap_chain_image_format: vk::Format,
    _swap_chain_extent: vk::Extent2D,
    swap_chain_image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    swap_chain_framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<Option<vk::Fence>>,
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
    current_frame: usize,
}

impl HelloTriangleApplication {
    pub fn new() -> HelloTriangleApplication {
        let (instance, entry, debug_utils, debug_messenger) = unsafe { Self::init_vulkan() };
        let surface = SurfaceKHR::null();
        let (physical_device, indices) = pick_physical_device(&instance, &entry, surface);
        let (device, graphics_queue, present_queue) =
            unsafe { create_logical_device(&instance, physical_device, indices.clone()) };
        let (swap_chain_ext, swap_chain, format, extent) = create_swap_chain(&instance, &entry, physical_device, surface, &device);
        let mut swap_chain_images = get_swap_chain_images(&instance, &device, swap_chain);
        let swap_chain_image_views = create_image_views(&mut swap_chain_images, format, &device);
        let render_pass = create_render_pass(format, &device);
        let (pipeline_layout, pipeline) = create_graphics_pipeline(&device, extent, render_pass);
        let swap_chain_framebuffers = create_framebuffers(&swap_chain_image_views, &device, render_pass, extent);
        let command_pool = create_command_pool(indices.clone(), &device);
        let command_buffers = create_command_buffers(&device, &swap_chain_framebuffers, command_pool, render_pass, extent, pipeline);
        let (image_available, render_finished, in_flight_fences, images_in_flight) = todo!();
        let surface_loader = khr::Surface::new(&entry, &instance);

        HelloTriangleApplication {
            instance,
            _entry: entry,
            debug_utils,
            debug_messenger,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            swap_chain_ext,
            swap_chain,
            _swap_chain_images: swap_chain_images,
            _swap_chain_image_format: format,
            _swap_chain_extent: extent,
            swap_chain_image_views,
            render_pass,
            pipeline_layout,
            pipeline,
            swap_chain_framebuffers,
            command_pool,
            command_buffers,
            image_available_semaphores: image_available,
            render_finished_semaphores: render_finished,
            in_flight_fences,
            images_in_flight,
            surface_loader,
            surface,
            current_frame: 0,
        }
    }

    pub unsafe fn init_vulkan(
    ) -> (
        Instance,
        Entry,
        Option<ext::DebugUtils>,
        Option<vk::DebugUtilsMessengerEXT>,
    ) {
        let app_name = CString::new("Hello Triangle").unwrap();
        let entry = Entry::new().unwrap();
        let extension_names = [];
        let layer_names = get_layer_names(&entry);

        let mut debug_messenger_info = get_debug_messenger_create_info();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .api_version(vk::make_version(1, 0, 0));
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names)
            .push_next(&mut debug_messenger_info);

        let instance = entry.create_instance(&create_info, None).unwrap();

        let (debug_utils, messenger) =
            setup_debug_messenger(&entry, &instance, &debug_messenger_info);

        (instance, entry, debug_utils, messenger)
    }

    pub fn draw_frame(&mut self) {
        let fence = self.in_flight_fences.get(self.current_frame).expect("Unable to get fence!");
        let fences = [*fence];

        unsafe { self.device.wait_for_fences(&fences, true, u64::MAX) }.expect("Unable to wait for fence");

        let image_available_semaphore = self.image_available_semaphores.get(self.current_frame).expect("Unable to get image_available semaphore for frame!");
        let render_finished_semaphore = self.render_finished_semaphores.get(self.current_frame).expect("Unable to get render_finished semaphore");

        let (image_index, _) = unsafe {
            self.swap_chain_ext.acquire_next_image(self.swap_chain, u64::MAX, *image_available_semaphore, vk::Fence::null()).expect("Unable to acquire image from swapchain")
        };

        if let Some(image_in_flight_fence) = unsafe { self.images_in_flight.get_unchecked(image_index as usize) } { 
            let fences = [*image_in_flight_fence];
            unsafe { self.device.wait_for_fences(&fences, true, u64::MAX) }.expect("Unable to wait for image_in_flight_fence");
        }

        self.images_in_flight[image_index as usize] = Some(*fence);

        println!("Drawing frame with index: {}", image_index);

        let wait_semaphores = [*image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let command_buffer = self.command_buffers.get(image_index as usize).unwrap();
        let command_buffers = [*command_buffer];

        let signal_semaphores = [*render_finished_semaphore];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .signal_semaphores(&signal_semaphores)
            .build();

        let submits = [submit_info];
        
        self.images_in_flight[image_index as usize] = None;
        unsafe { self.device.reset_fences(&fences) }.expect("Unable to reset fences");
        unsafe { self.device.queue_submit(self._graphics_queue, &submits, *fence).expect("Unable to submit to queue") };

        let swap_chains = [self.swap_chain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(&swap_chains)
            .wait_semaphores(&signal_semaphores)
            .image_indices(&image_indices);

        unsafe { self.swap_chain_ext.queue_present(self._present_queue, &present_info).expect("Unable to present") };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        unsafe {
            for semaphore in self.render_finished_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }

            for semaphore in self.image_available_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }

            for fence in self.in_flight_fences.drain(..) {
                self.device.destroy_fence(fence, None);
            }

            self.device.destroy_command_pool(self.command_pool, None);
            // WARNING: self.command_pool is now invalid!

            for framebuffer in self.swap_chain_framebuffers.drain(..) {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            // self.swap_chain_framebuffers will now be empty

            for view in self.swap_chain_image_views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            // self.swap_chain_image_views will now be empty

            self.device.destroy_render_pass(self.render_pass, None);

            let swapchain = khr::Swapchain::new(&self.instance, &self.device);
            swapchain.destroy_swapchain(self.swap_chain, None);
            // WARNING: self.swap_chain is now invalid!

            self.debug_messenger.map(|m| {
                self.debug_utils.as_ref().map(|d| {
                    d.destroy_debug_utils_messenger(m, None);
                })
            });
            // WARNING: self.debug_messenger is now invalid!

            self.surface_loader.destroy_surface(self.surface, None);
            // WARNING: self.surface is now invalid!

            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            // WARNING: self.pipeline_layout is now invalid!

            self.device.destroy_pipeline(self.pipeline, None);
            // WARNING: self.pipeline is now invalid!


            self.device.destroy_device(None);
            // WARNING: self.device is now invalid!

            self.instance.destroy_instance(None);
            // WARNING: self.instance is now invalid!
        }
    }
}

fn main() {
    let app = HelloTriangleApplication::new();
}
pub struct SyncObjects {
    pub in_flight_fences: Vec<vk::Fence>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub images_in_flight: Vec<Option<vk::Fence>>,
}

// Semaphores
pub fn create_sync_objects(device: &Device, swapchain_images_size: usize) -> SyncObjects {
    let mut image_available_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    let mut render_finished_semaphores = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
    let mut images_in_flight = Vec::with_capacity(swapchain_images_size);

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);
    
    for _ in 0..MAX_FRAMES_IN_FLIGHT {

        let image_available = unsafe { device.create_semaphore(&semaphore_info, None).expect("Unable to create semaphore") };
        image_available_semaphores.push(image_available);

        let render_finished = unsafe { device.create_semaphore(&semaphore_info, None).expect("Unable to create semaphore") };
        render_finished_semaphores.push(render_finished);

        let in_flight_fence = unsafe { device.create_fence(&fence_info, None)}.expect("Unable to create fence!");
        in_flight_fences.push(in_flight_fence);
    }

    println!("swapchain images size: {}", swapchain_images_size);
    for _ in 0..swapchain_images_size {
        images_in_flight.push(None);
    }

    SyncObjects {
        image_available_semaphores, render_finished_semaphores, in_flight_fences, images_in_flight
    }
}

// Command Buffers/Pools
fn create_command_pool(queue_family_indices: QueueFamilyIndices, device: &Device) -> vk::CommandPool {
    let pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family.unwrap());

    unsafe { device.create_command_pool(&pool_info, None).expect("Unable to create command pool") }
}

pub fn create_command_buffers(device: &Device, swap_chain_framebuffers: &Vec<vk::Framebuffer>, command_pool: vk::CommandPool, render_pass: vk::RenderPass, extent: vk::Extent2D, graphics_pipeline: vk::Pipeline) -> Vec<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(swap_chain_framebuffers.len() as u32);
    
    let command_buffers = unsafe { device.allocate_command_buffers(&alloc_info).expect("Unable to allocate frame_buffers") };

    for (command_buffer, framebuffer) in command_buffers.iter().zip(swap_chain_framebuffers) {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe { device.begin_command_buffer(*command_buffer, &begin_info).expect("Unable to begin command buffer"); }
            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0},
                extent,
            };
        
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0]
            }
        };

        let clear_colors = [clear_color];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(*framebuffer)
            .render_area(render_area)
            .clear_values(&clear_colors);

        unsafe { 
            device.cmd_begin_render_pass(*command_buffer, &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);
            device.cmd_draw(*command_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(*command_buffer);
            device.end_command_buffer(*command_buffer).expect("Unable to record command buffer!");
        }
    }

    command_buffers
}

// Graphics Pipeline
pub fn create_graphics_pipeline(device: &Device, extent: vk::Extent2D, render_pass: vk::RenderPass) -> (vk::PipelineLayout, vk::Pipeline) { 
    let vert_shader_code = include_bytes!("./shaders/shader.vert.spv");
    let frag_shader_code = include_bytes!("./shaders/shader.frag.spv");

    let vertex_shader_module = create_shader_module(device, vert_shader_code);
    let frag_shader_module = create_shader_module(device, frag_shader_code);
    let name = CString::new("main").unwrap();

    let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertex_shader_module)
        .name(name.as_c_str())
        .build();

    let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_shader_module)
        .name(name.as_c_str())
        .build();
    
    let shader_stages = [vertex_shader_stage_info, frag_shader_stage_info]; 

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
    let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(extent.width as f32)
        .height(extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0)
        .build();

    let viewports = [viewport];
    
    let offset = vk::Offset2D { x: 0, y: 0}; 
    let scissor = vk::Rect2D::builder()
        .offset(offset)
        .extent(extent)
        .build();

    let scissors = [scissor];
    
    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewport_count(1)
        .viewports(&viewports)
        .scissor_count(1)
        .scissors(&scissors);
    
    let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);
    
    let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .min_sample_shading(1.0);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
        .blend_enable(false)
        .build();

    let color_blend_attachments = [color_blend_attachment];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&color_blend_attachments);

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
    let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None).unwrap() };

    let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_create_info)
        .viewport_state(&viewport_state_create_info)
        .rasterization_state(&rasterizer_create_info)
        .multisample_state(&multisampling_create_info)
        .color_blend_state(&color_blend_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0)
        .build();

    let create_infos = [pipeline_create_info];
    
    let mut graphics_pipelines = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &create_infos, None).unwrap() };

    // Cleanup
    unsafe { device.destroy_shader_module(vertex_shader_module, None) } ;
    unsafe { device.destroy_shader_module(frag_shader_module, None) } ;

    return (pipeline_layout, graphics_pipelines.remove(0));
}

fn create_shader_module(device: &Device, bytes: &[u8]) -> vk::ShaderModule {
    let (_, code, _) = unsafe { bytes.align_to::<u32>() };
    let create_info = vk::ShaderModuleCreateInfo::builder()
        .code(code);

    unsafe { device.create_shader_module(&create_info, None).expect("Unable to create shader module") }
}

pub fn create_render_pass(format: vk::Format, device: &Device) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(format)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build();

    let color_attachments = [color_attachment];

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let color_attachment_refs = [color_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&color_attachment_refs)
        .build();
    let subpasses = [subpass];

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .build();
    let dependencies = [dependency];

    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe { device.create_render_pass(&render_pass_create_info, None).unwrap() }
}

// Logical Device
unsafe fn create_logical_device<'a>(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    indices: QueueFamilyIndices,
) -> (Device, vk::Queue, vk::Queue) {
    let required_extensions = vec![khr::Swapchain::name()];

    // TODO: Portability
    // let extensions = portability_extensions();
    // if has_portability(instance, physical_device) {
    //     let mut extensions = extensions.iter().map(|i| i.as_c_str()).collect();
    //     required_extensions.append(&mut extensions);
    // }
    let required_extensions_raw = required_extensions.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
    let queue_priorities = [1.0];
    let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_priorities(&queue_priorities)
        .queue_family_index(indices.graphics_family.unwrap())
        .build();


    let mut queue_create_infos = vec![graphics_queue_create_info];

    if !indices.are_same() {
        let present_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_priorities(&queue_priorities)
            .queue_family_index(indices.present_family.unwrap())
            .build();
        queue_create_infos.push(present_queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures::builder();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_create_infos[..])
        .enabled_extension_names(&required_extensions_raw)
        .enabled_features(&physical_device_features);

    let device = instance
        .create_device(physical_device, &device_create_info, None)
        .unwrap();

    let graphics_queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
    let present_queue = device.get_device_queue(indices.present_family.unwrap(), 0);

    (device, graphics_queue, present_queue)
}

// Surface
fn get_required_extensions_for_window(window: &Window, entry: &Entry) -> Vec<*const i8> {
    // let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();

    // let supported_extensions = entry
    //     .enumerate_instance_extension_properties()
    //     .expect("Unable to enumerate instance extension properties")
    //     .iter()
    //     .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
    //     .collect::<Vec<_>>();

    let extension_names_raw = Vec::new();
    // for extension in surface_extensions {
    //     assert!(
    //         supported_extensions.contains(&extension),
    //         "Unsupported extension: {:?}",
    //         extension
    //     );
    //     extension_names_raw.push(extension.as_ptr())
    // }

    // if should_add_validation_layers() {
    //     let debug_utils = ext::DebugUtils::name();
    //     extension_names_raw.push(debug_utils.as_ptr())
    // }

    return extension_names_raw;
}

// Swap Chain
fn create_swap_chain (
    instance: &Instance,
    entry: &Entry,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    logical_device: &Device) -> (khr::Swapchain, vk::SwapchainKHR, vk::Format, vk::Extent2D) {
    let swap_chain_support = SwapChainSupportDetails::query_swap_chain_support(instance, entry, physical_device, surface);

    let surface_format = choose_swap_surface_format(swap_chain_support.surface_formats);
    let present_mode = choose_swap_present_mode(swap_chain_support.present_modes);
    let extent = choose_swap_extent(swap_chain_support.capabilities);

    let image_count = swap_chain_support.capabilities.min_image_count + 1;
    let image_count = if swap_chain_support.capabilities.max_image_count > 0 && image_count > swap_chain_support.capabilities.max_image_count {
        swap_chain_support.capabilities.max_image_count
    } else {
        image_count
    };

    let indices = QueueFamilyIndices::find_queue_families(instance, physical_device, entry, surface);
    let indices_are_same = indices.are_same();
    let image_sharing_mode = if indices_are_same { vk::SharingMode::EXCLUSIVE } else { vk::SharingMode::CONCURRENT };
    let queue_family_indices = if indices_are_same { Vec::new() } else { vec![indices.graphics_family.unwrap(), indices.present_family.unwrap()] };

    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(swap_chain_support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .old_swapchain(vk::SwapchainKHR::null())
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let swapchain_ext = khr::Swapchain::new(instance, logical_device);
    let swapchain = unsafe { swapchain_ext.create_swapchain(&create_info, None) }.expect("Unable to create Swapchain");
    (swapchain_ext, swapchain, surface_format.format, extent)
}

fn choose_swap_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
    for available_format in &formats { 
        if available_format.format == vk::Format::B8G8R8A8_SRGB && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return *available_format
        }
    }

    return *formats.first().unwrap();
}

fn choose_swap_present_mode(available_present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for available_present_mode in &available_present_modes {
        if available_present_mode == &vk::PresentModeKHR::MAILBOX {
            return *available_present_mode
        }
    }
    return *available_present_modes.first().unwrap();
}

fn choose_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    return capabilities.current_extent
}

fn get_swap_chain_images(instance: &Instance, device: &Device, swap_chain: vk::SwapchainKHR) -> Vec<vk::Image> {
    let swap_chain_ext = khr::Swapchain::new(instance, device);
    unsafe { swap_chain_ext.get_swapchain_images(swap_chain).expect("Unable to get swapchain images") }
}

fn create_image_views(swap_chain_images: &mut Vec<vk::Image>, format: vk::Format, device: &Device) -> Vec<vk::ImageView> {
    swap_chain_images.drain(..).map(|image| {
        let components = vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY)
            .build();

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(components)
            .subresource_range(subresource_range);

        unsafe { device.create_image_view(&create_info, None).expect("Unable to get image view") }
    }).collect::<Vec<_>>()
}

pub fn create_framebuffers(image_views: &Vec<vk::ImageView>, device: &Device, render_pass: vk::RenderPass, extent: vk::Extent2D) -> Vec<vk::Framebuffer> {
    image_views.iter().map(|v| {
        let attachments = [*v]; //.. really?
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        
        unsafe { device.create_framebuffer(&create_info, None).unwrap() }
    })
    .collect::<Vec<_>>()
}


// Physical Device
fn pick_physical_device(
    instance: &Instance,
    entry: &Entry,
    surface: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, QueueFamilyIndices) {
    unsafe {
        let devices = instance.enumerate_physical_devices().unwrap();
        let mut devices = devices.into_iter().map(|d| {
            get_suitability(d, instance, entry, surface)
        }).collect::<Vec<_>>();
        devices.sort_by_key(|i| i.0);

        let (_, indices, device) = devices.remove(0);
        (device, indices)
    }
}

/// Gets a device's suitability. Lower score is bettter.
unsafe fn get_suitability(
    device: vk::PhysicalDevice,
    instance: &Instance,
    entry: &Entry,
    surface: vk::SurfaceKHR,
) -> (i8, QueueFamilyIndices, vk::PhysicalDevice) {
    let properties = instance.get_physical_device_properties(device);
    let indices = QueueFamilyIndices::find_queue_families(instance, device, entry, surface);
    let has_extension_support = check_device_extension_support(instance, device);
    let swap_chain_adequate = check_swap_chain_adequate(instance, entry, surface, device);
    let has_graphics_family = indices.graphics_family.is_some();

    let mut suitability = 0;
    if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
        suitability -= 5;
    }

    let suitable = has_extension_support && swap_chain_adequate && has_graphics_family;

    if suitable { suitability -= 1 }

    (suitability, indices, device)
}

fn check_swap_chain_adequate(instance: &Instance, entry: &Entry, surface: vk::SurfaceKHR, device: vk::PhysicalDevice) -> bool {
    let swap_chain_support_details = SwapChainSupportDetails::query_swap_chain_support(instance, entry, device, surface);
    !swap_chain_support_details.surface_formats.is_empty() && !swap_chain_support_details.present_modes.is_empty()
}

fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
    let extensions = unsafe { instance.enumerate_device_extension_properties(device).expect("Unable to get extension properties") }
        .iter()
        .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
        .collect::<Vec<_>>();

        let required_extension = khr::Swapchain::name();

        extensions.contains(&required_extension)
}

// TODO: Portability?
// fn has_portability(instance: &Instance, device: vk::PhysicalDevice) -> bool {
//     let extensions = unsafe { instance.enumerate_device_extension_properties(device).expect("Unable to get extension properties") }
//         .iter()
//         .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
//         .collect::<Vec<_>>();

//         let portability_extension = CString::new("VK_KHR_portability_subset").unwrap();
//         extensions.contains(&portability_extension.as_c_str())
// } 

// fn portability_extensions() -> Vec<CString> {
//     vec![
//         CString::new("VK_KHR_portability_subset").unwrap(),
//         CString::new("VK_KHR_get_physical_device_properties2").unwrap()
//     ]
// }

// Debug Messenger

fn get_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
    let message_severity = 
    // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
    let message_type = vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(message_severity)
        .message_type(message_type)
        .pfn_user_callback(Some(debug_messenger_callback))
}

#[cfg(debug_assertions)]
fn setup_debug_messenger(
    entry: &Entry,
    instance: &Instance,
    info: &vk::DebugUtilsMessengerCreateInfoEXT,
) -> (Option<ext::DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    let debug_utils = ext::DebugUtils::new(entry, instance);

    let messenger = unsafe {
        debug_utils
            .create_debug_utils_messenger(info, None)
            .unwrap()
    };

    (Some(debug_utils), Some(messenger))
}

#[cfg(not(debug_assertions))]
fn setup_debug_messenger(
    entry: &Entry,
    instance: &Instance,
) -> (Option<ext::DebugUtils>, Option<vk::DebugUtilsMessengerEXT>) {
    (None, None)
}

fn get_layer_names(entry: &Entry) -> Vec<*const u8> {
    let mut validation_layers_raw = Vec::new();
    if !should_add_validation_layers() {
        return validation_layers_raw;
    };

    let validation_layers = get_validation_layers();
    let supported_layers = entry
        .enumerate_instance_layer_properties()
        .expect("Unable to enumerate instance layer properties")
        .iter()
        .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
        .collect::<Vec<_>>();

    for layer in validation_layers {
        assert!(
            supported_layers.contains(&layer),
            "Unsupported layer: {:?}",
            layer
        );
        validation_layers_raw.push(layer.as_ptr())
    }

    return validation_layers_raw;
}

#[cfg(debug_assertions)]
unsafe extern "system" fn debug_messenger_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    println!(
        "[VULKAN]: {:?}",
        CStr::from_ptr((*p_callback_data).p_message)
    );
    return vk::FALSE;
}

#[cfg(debug_assertions)]
fn get_validation_layers() -> Vec<&'static CStr> {
    let validation_layer = CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap();
    return vec![validation_layer];
}

#[cfg(not(debug_assertions))]
fn get_validation_layers() -> Vec<&'static CStr> {
    return Vec::new();
}

#[cfg(debug_assertions)]
fn should_add_validation_layers() -> bool {
    true
}

#[cfg(not(debug_assertions))]
fn should_add_validation_layers() -> bool {
    false
}