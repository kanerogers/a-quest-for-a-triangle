use crate::old_vulkan;
use ash::{version::DeviceV1_0, vk, Device};
use ovr_mobile_sys::ovrVector4f;

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
    pub clear_color: ovrVector4f,
    pub colour_format: vk::Format,
    pub depth_format: vk::Format,
    pub sample_count: vk::SampleCountFlags,
}

impl RenderPass {
    pub fn new(device: &Device) -> Self {
        let colour_format = vk::Format::R8G8B8A8_UNORM;
        let depth_format = vk::Format::D24_UNORM_S8_UINT;
        let sample_count = vk::SampleCountFlags::TYPE_1;
        let render_pass = create_render_pass(device, colour_format, depth_format, sample_count);
        let clear_color = ovrVector4f {
            x: 0.125,
            y: 0.0,
            z: 0.125,
            w: 1.0,
        };

        Self {
            render_pass,
            clear_color,
            colour_format,
            depth_format,
            sample_count,
        }
    }
}

pub fn create_render_pass(
    device: &Device,
    colour_format: vk::Format,
    depth_format: vk::Format,
    sample_count: vk::SampleCountFlags,
) -> vk::RenderPass {
    println!("[RenderPass] Creating render pass..");

    let color_attachment = vk::AttachmentDescription::builder()
        .format(colour_format)
        .samples(sample_count)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();

    let depth_stencil_attachment = vk::AttachmentDescription::builder()
        .format(depth_format)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();
    let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_attachment_ref])
        .depth_stencil_attachment(&depth_stencil_attachment_ref)
        .build();
    let subpasses = [subpass];

    let attachments = [
        color_attachment,
        // fragment_density_attachment, // TODO: FFR
        depth_stencil_attachment,
    ];

    // TODO: Mutli View
    // let view_mask = [0b00000011];
    // let mut multiview_create_info = vk::RenderPassMultiviewCreateInfo::builder()
    //     .view_masks(&view_mask)
    //     .correlation_masks(&view_mask);

    // TODO: FFR
    // let mut fragment_density_map_create_info =
    //     vk::RenderPassFragmentDensityMapCreateInfoEXT::builder()
    //         .fragment_density_map_attachment(fragment_density_attachment_ref);

    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        // .push_next(&mut multiview_create_info)
        // .push_next(&mut fragment_density_map_create_info)
        .subpasses(&subpasses);

    let render_pass = unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .expect("Unable to create Render Pass")
    };

    println!("[RenderPass] ..done!");
    return render_pass;
}
