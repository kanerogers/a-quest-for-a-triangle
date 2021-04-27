use crate::{
    debug_messenger::{get_debug_messenger_create_info, setup_debug_messenger},
    device::create_logical_device,
    physical_device::get_physical_device,
    util::cstrings_to_raw,
};
use ash::{
    version::{DeviceV1_0, EntryV1_0, InstanceV1_0},
    vk::{self, Handle},
    Device, Entry, Instance,
};
use ovr_mobile_sys::{
    ovrSystemCreateInfoVulkan, vrapi_CreateSystemVulkan, vrapi_GetDeviceExtensionsVulkan,
    vrapi_GetInstanceExtensionsVulkan, VkDevice_T, VkInstance_T, VkPhysicalDevice_T,
};
use std::ffi::{CStr, CString};

#[derive(Clone)]
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub command_pool: vk::CommandPool,
    pub pipeline_cache: vk::PipelineCache,
}

impl std::fmt::Debug for VulkanContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanContext")
            .field("physical_device", &self.physical_device)
            .finish()
    }
}

impl VulkanContext {
    pub fn new() -> Self {
        let (instance, entry) = vulkan_init();
        let required_device_extensions = get_required_device_extensions();

        let (physical_device, queue_family_indices) =
            get_physical_device(&instance, &required_device_extensions);

        let (device, graphics_queue, present_queue) = create_logical_device(
            &instance,
            physical_device,
            &queue_family_indices,
            &required_device_extensions,
        );

        let command_pool =
            create_command_pool(&device, queue_family_indices.graphics_family.unwrap());
        let pipeline_cache = create_pipeline_cache(&device);

        create_system_vulkan(&instance, physical_device, &device);

        Self {
            entry,
            instance,
            device,
            physical_device,
            graphics_queue,
            present_queue,
            command_pool,
            pipeline_cache,
        }
    }

    pub fn change_image_layout(
        &self,
        command_buffer: vk::CommandBuffer,
        image: &vk::Image,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
    ) {
        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let image_memory_barrier = vk::ImageMemoryBarrier::builder()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .image(*image)
            .subresource_range(subresource_range)
            .build();

        let dependency_flags = vk::DependencyFlags::empty();
        let image_memory_barriers = [image_memory_barrier];

        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                &[],
                &[],
                &image_memory_barriers,
            )
        };
    }

    pub fn create_image(
        &self,
        width: i32,
        height: i32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> vk::Image {
        let device = &self.device;
        println!("[VulkanContext] Creating image..");

        let face_count = 1;
        let num_storage_levels = 1;
        let array_layers_count = face_count;
        let sample_count = vk::SampleCountFlags::TYPE_1;
        let extent = vk::Extent3D::builder()
            .width(width as u32)
            .height(height as u32)
            .depth(1)
            .build();

        let create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(num_storage_levels)
            .array_layers(array_layers_count)
            .samples(sample_count)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            device
                .create_image(&create_info, None)
                .expect("Unable to create render image")
        };

        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };
        let memory_type = memory_requirements.memory_type_bits;
        let memory_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let memory_type_index = self.get_memory_type_index(memory_type, memory_flags);
        let allocation_size = memory_requirements.size;
        let memory_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(allocation_size)
            .memory_type_index(memory_type_index);

        println!("[VulkanContext] Creating device memory..");
        let device_memory = unsafe {
            device
                .allocate_memory(&memory_allocate_info, None)
                .expect("Unable to allocate memory")
        };
        println!("[VulkanContext] ..done. Binding memory..");
        unsafe {
            device
                .bind_image_memory(image, device_memory, 0)
                .expect("Unable to bind image memory")
        };

        println!("[VulkanContext] ..done. created image: {:?}", image);
        image
    }

    pub fn create_image_view(
        &self,
        image: &vk::Image,
        color_format: vk::Format,
        aspect_mask: vk::ImageAspectFlags,
    ) -> vk::ImageView {
        let components = vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::R)
            .g(vk::ComponentSwizzle::G)
            .b(vk::ComponentSwizzle::B)
            .a(vk::ComponentSwizzle::A)
            .build();

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .level_count(1)
            .layer_count(1)
            .build();

        let create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(color_format)
            .components(components)
            .subresource_range(subresource_range);

        unsafe {
            self.device
                .create_image_view(&create_info, None)
                .expect("Unable to create image view")
        }
    }

    fn get_memory_type_index(
        &self,
        required_memory_type_bits: u32,
        required_memory_flags: vk::MemoryPropertyFlags,
    ) -> u32 {
        let properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        for memory_index in 0..properties.memory_type_count {
            let memory_type_bits = 1 << memory_index;
            let is_required_memory_type = required_memory_type_bits & memory_type_bits == 1;
            let memory_flags = properties.memory_types[memory_index as usize].property_flags;
            let has_required_properties = memory_flags.contains(required_memory_flags);

            if is_required_memory_type && has_required_properties {
                return memory_index;
            }
        }
        panic!("Unable to find suitable memory type index");
    }

    pub fn create_setup_command_buffer(&self) -> vk::CommandBuffer {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let buffer = unsafe {
            self.device
                .allocate_command_buffers(&allocate_info)
                .expect("Unable to allocate command buffer")
                .pop()
                .unwrap()
        };

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(buffer, &begin_info)
                .expect("Unable to begin command buffer")
        };

        println!("[VulkanContext] Created setup command buffer {:?}", buffer);

        return buffer;
    }

    pub fn flush_setup_command_buffer(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .expect("Unable to end command buffer")
        };
        let command_buffers = &[command_buffer];

        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(command_buffers)
            .build();

        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], vk::Fence::null())
                .expect("Failed to submit queue");
            self.device
                .queue_wait_idle(self.graphics_queue)
                .expect("Failed to wait for queue to be idle");
            self.device
                .free_command_buffers(self.command_pool, command_buffers)
        };
    }
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    println!("[VulkanContext] Creating command pool");
    let create_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family_index)
        .build();

    let command_pool = unsafe {
        device
            .create_command_pool(&create_info, None)
            .expect("Unable to create command pool")
    };

    println!("[VulkanContext] Done");
    return command_pool;
}

fn create_pipeline_cache(device: &Device) -> vk::PipelineCache {
    println!("[VulkanContext] Creating pipeline cache");
    let create_info =
        vk::PipelineCacheCreateInfo::builder().flags(vk::PipelineCacheCreateFlags::empty());

    let pipeline_cache = unsafe {
        device
            .create_pipeline_cache(&create_info, None)
            .expect("Unable to create pipeline cache")
    };

    println!("[VulkanContext] Done");

    return pipeline_cache;
}

fn create_system_vulkan(instance: &Instance, physical_device: vk::PhysicalDevice, device: &Device) {
    let vk_instance = instance.handle().as_raw();
    let vk_physical_device = physical_device.as_raw();
    let vk_device = device.handle().as_raw();
    let mut system_info = ovrSystemCreateInfoVulkan {
        Instance: vk_instance as *mut VkInstance_T,
        PhysicalDevice: vk_physical_device as *mut VkPhysicalDevice_T,
        Device: vk_device as *mut VkDevice_T,
    };
    println!("[VulkanContext] Calling vrapi_CreateSystemVulkan..");
    unsafe { vrapi_CreateSystemVulkan(&mut system_info) };
    println!("[VulkanContext] ..done. VulkanContext created!");
}

fn vulkan_init() -> (Instance, Entry) {
    println!("[VulkanContext] Initialising Vulkan..");
    let app_name = CString::new("A Quest for a Triangle").unwrap();
    let entry = Entry::new().unwrap();
    let layer_names = get_layer_names(&entry);
    let layer_names_raw = cstrings_to_raw(&layer_names);

    let mut debug_messenger_info = get_debug_messenger_create_info();
    let extension_names = get_instance_extensions();
    let extension_names_raw = cstrings_to_raw(&extension_names);

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .api_version(vk::make_version(1, 0, 0));
    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names_raw)
        .enabled_layer_names(&layer_names_raw)
        .push_next(&mut debug_messenger_info);

    let instance = unsafe { entry.create_instance(&create_info, None) }.unwrap();
    let (_debug_utils, _messenger) =
        setup_debug_messenger(&entry, &instance, &debug_messenger_info);

    println!("[VulkanContext] ..done");

    (instance, entry)
}

#[cfg(debug_assertions)]
fn get_instance_extensions() -> Vec<CString> {
    let instance_extension_names = CString::new("").unwrap();
    let p = instance_extension_names.into_raw();
    unsafe { vrapi_GetInstanceExtensionsVulkan(p, &mut 4096) };
    let instance_extension_names = unsafe { CString::from_raw(p) };

    let mut extensions = instance_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();

    extensions.push(vk::KhrGetPhysicalDeviceProperties2Fn::name().to_owned());
    extensions.push(vk::ExtDebugUtilsFn::name().to_owned());

    return extensions;
}

#[cfg(not(debug_assertions))]
fn get_extension_names() -> Vec<&'static CStr> {
    return Vec::new();
}

fn get_required_device_extensions() -> Vec<CString> {
    let device_extension_names = CString::new("").unwrap();
    let p = device_extension_names.into_raw();
    unsafe { vrapi_GetDeviceExtensionsVulkan(p, &mut 4096) };
    let device_extension_names = unsafe { CString::from_raw(p) };

    let names = device_extension_names
        .to_str()
        .unwrap()
        .split(" ")
        .map(|c| CString::new(c).unwrap())
        .collect::<Vec<_>>();

    return names;
}

fn get_layer_names(entry: &Entry) -> Vec<CString> {
    let validation_layers = get_validation_layers();
    let supported_layers = entry
        .enumerate_instance_layer_properties()
        .expect("Unable to enumerate instance layer properties")
        .iter()
        .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
        .collect::<Vec<_>>();

    for layer in &validation_layers {
        assert!(supported_layers.contains(&layer.as_c_str()));
    }

    return validation_layers;
}

#[cfg(debug_assertions)]
fn get_validation_layers() -> Vec<CString> {
    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    return vec![validation_layer];
}

#[cfg(not(debug_assertions))]
fn get_validation_layers() -> Vec<CString> {
    return Vec::new();
}
