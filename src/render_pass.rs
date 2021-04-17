use ash::{version::DeviceV1_0, vk, Device};

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

    unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .unwrap()
    }
}
