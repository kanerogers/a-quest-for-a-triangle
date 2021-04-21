use ash::{version::DeviceV1_0, vk, Device};
use ovr_mobile_sys::ovrVector4f;

pub enum RenderPassType {
    Inline,
    SecondaryCommandBuffers,
}

pub struct RenderPass {
    pub render_pass_type: RenderPassType,
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(device: &Device) -> Self {
        let render_pass = create_render_pass(device);
        Self {
            render_pass,
            render_pass_type: RenderPassType::Inline,
        }
    }
}

pub fn create_render_pass(device: &Device) -> vk::RenderPass {
    println!("[RenderPass] Creating render pass");
    let colour_format = vk::Format::R8G8B8A8_UNORM;
    let depth_format = vk::Format::D24_UNORM_S8_UINT;
    let sample_count = vk::SampleCountFlags::TYPE_4;
    let clear_colour = ovrVector4f {
        x: 0.125,
        y: 0.0,
        z: 0.125,
        w: 1.0,
    };

    let color_attachment = vk::AttachmentDescription::builder()
        .format(colour_format)
        .samples(sample_count)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let color_attachments = [color_attachment];
    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let color_attachment_refs = [color_attachment_ref];

    let resolve_attachment = vk::AttachmentDescription::builder()
        .format(colour_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let resolve_attachments = [resolve_attachment];
    let resolve_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .build();
    let resolve_attachment_refs = [resolve_attachment_ref];

    let depth_attachment = vk::AttachmentDescription::builder()
        .format(depth_format)
        .samples(sample_count)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT)
        .final_layout(vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT)
        .build();
    let depth_attachments = [depth_attachment];
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .build();
    let depth_attachment_refs = [depth_attachment_ref];

    let fragment_density_attachment = vk::AttachmentDescription::builder()
        .format(vk::Format::R8G8_UNORM)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT)
        .final_layout(vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT)
        .build();
    let fragment_density_attachments = [fragment_density_attachment];
    let fragment_density_attachment_ref = vk::AttachmentReference::builder()
        .attachment(3)
        .layout(vk::ImageLayout::FRAGMENT_DENSITY_MAP_OPTIMAL_EXT)
        .build();
    let fragment_density_attachment_refs = [fragment_density_attachment_ref];

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

    let render_pass = unsafe {
        device
            .create_render_pass(&render_pass_create_info, None)
            .expect("Unable to create Render Pass")
    };

    println!("[RenderPass] ..done!");
    return render_pass;
}

// typedef struct {
//     ovrVkRenderPassType type;
//     int flags;
//     ovrSurfaceColorFormat colorFormat;
//     ovrSurfaceDepthFormat depthFormat;
//     ovrSampleCount sampleCount;
//     VkFormat internalColorFormat;
//     VkFormat internalDepthFormat;
//     VkFormat internalFragmentDensityFormat;
//     VkRenderPass renderPass;
//     ovrVector4f clearColor;
// } ovrVkRenderPass;

// int flags = OVR_RENDERPASS_FLAG_CLEAR_COLOR_BUFFER | OVR_RENDERPASS_FLAG_CLEAR_DEPTH_BUFFER;
// if (useFFR) {
//     flags |= OVR_RENDERPASS_FLAG_INCLUDE_FRAG_DENSITY;
// }

/*
ovrVkRenderPass_Create(
    context,
    &renderer->RenderPassSingleView,
    OVR_SURFACE_COLOR_FORMAT_R8G8B8A8,
    OVR_SURFACE_DEPTH_FORMAT_D24,
    SAMPLE_COUNT,
    OVR_RENDERPASS_TYPE_INLINE,
    flags,
    &clearColor,
    isMultiview);
    */

//     bool ovrVkRenderPass_Create(
//     ovrVkContext* context,
//     ovrVkRenderPass* renderPass,
//     const ovrSurfaceColorFormat colorFormat,
//     const ovrSurfaceDepthFormat depthFormat,
//     const ovrSampleCount sampleCount,
//     const ovrVkRenderPassType type,
//     const int flags,
//     const ovrVector4f* clearColor,
//     bool isMultiview) {
//     assert(
//         (context->device->physicalDeviceProperties.properties.limits.framebufferColorSampleCounts &
//          (VkSampleCountFlags)sampleCount) != 0);
//     assert(
//         (context->device->physicalDeviceProperties.properties.limits.framebufferDepthSampleCounts &
//          (VkSampleCountFlags)sampleCount) != 0);

//     renderPass->type = type;
//     renderPass->flags = flags;
//     renderPass->colorFormat = colorFormat;
//     renderPass->depthFormat = depthFormat;
//     renderPass->sampleCount = sampleCount;
//     renderPass->internalColorFormat = ovrGpuColorBuffer_InternalSurfaceColorFormat(colorFormat);
//     renderPass->internalDepthFormat = ovrGpuDepthBuffer_InternalSurfaceDepthFormat(depthFormat);
//     renderPass->internalFragmentDensityFormat = VK_FORMAT_R8G8_UNORM;
//     renderPass->clearColor = *clearColor;

//     uint32_t attachmentCount = 0;
//     VkAttachmentDescription attachments[4];

//     // Optionally use a multi-sampled attachment.
//     if (sampleCount > OVR_SAMPLE_COUNT_1) {
//         attachments[attachmentCount].flags = 0;
//         attachments[attachmentCount].format = renderPass->internalColorFormat;
//         attachments[attachmentCount].samples = (VkSampleCountFlagBits)sampleCount;
//         attachments[attachmentCount].loadOp =
//             ((flags & OVR_RENDERPASS_FLAG_CLEAR_COLOR_BUFFER) != 0)
//             ? VK_ATTACHMENT_LOAD_OP_CLEAR
//             : VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].storeOp = (EXPLICIT_RESOLVE != 0)
//             ? VK_ATTACHMENT_STORE_OP_STORE
//             : VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].initialLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
//         attachments[attachmentCount].finalLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
//         attachmentCount++;
//     }
//     // Either render directly to, or resolve to the single-sample attachment.
//     if (sampleCount <= OVR_SAMPLE_COUNT_1 || EXPLICIT_RESOLVE == 0) {
//         attachments[attachmentCount].flags = 0;
//         attachments[attachmentCount].format = renderPass->internalColorFormat;
//         attachments[attachmentCount].samples = VK_SAMPLE_COUNT_1_BIT;
//         attachments[attachmentCount].loadOp =
//             ((flags & OVR_RENDERPASS_FLAG_CLEAR_COLOR_BUFFER) != 0 &&
//              sampleCount <= OVR_SAMPLE_COUNT_1)
//             ? VK_ATTACHMENT_LOAD_OP_CLEAR
//             : VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].storeOp = VK_ATTACHMENT_STORE_OP_STORE;
//         attachments[attachmentCount].stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].initialLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
//         attachments[attachmentCount].finalLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
//         attachmentCount++;
//     }
//     // Optionally use a depth buffer.
//     if (renderPass->internalDepthFormat != VK_FORMAT_UNDEFINED) {
//         attachments[attachmentCount].flags = 0;
//         attachments[attachmentCount].format = renderPass->internalDepthFormat;
//         attachments[attachmentCount].samples = (VkSampleCountFlagBits)sampleCount;
//         attachments[attachmentCount].loadOp =
//             ((flags & OVR_RENDERPASS_FLAG_CLEAR_DEPTH_BUFFER) != 0)
//             ? VK_ATTACHMENT_LOAD_OP_CLEAR
//             : VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].storeOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].initialLayout =
//             VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
//         attachments[attachmentCount].finalLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
//         attachmentCount++;
//     }

//     uint32_t sampleMapAttachment = 0;
//     if ((flags & OVR_RENDERPASS_FLAG_INCLUDE_FRAG_DENSITY) != 0) {
//         sampleMapAttachment = attachmentCount;
//         attachments[attachmentCount].flags = 0;
//         attachments[attachmentCount].format = renderPass->internalFragmentDensityFormat;
//         attachments[attachmentCount].samples = VK_SAMPLE_COUNT_1_BIT;
//         attachments[attachmentCount].loadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].storeOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].stencilLoadOp = VK_ATTACHMENT_LOAD_OP_DONT_CARE;
//         attachments[attachmentCount].stencilStoreOp = VK_ATTACHMENT_STORE_OP_DONT_CARE;
//         attachments[attachmentCount].initialLayout =
//             VK_IMAGE_LAYOUT_FRAGMENT_DENSITY_MAP_OPTIMAL_EXT;
//         attachments[attachmentCount].finalLayout = VK_IMAGE_LAYOUT_FRAGMENT_DENSITY_MAP_OPTIMAL_EXT;
//         attachmentCount++;
//     }

//     VkAttachmentReference colorAttachmentReference;
//     colorAttachmentReference.attachment = 0;
//     colorAttachmentReference.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;

//     VkAttachmentReference resolveAttachmentReference;
//     resolveAttachmentReference.attachment = 1;
//     resolveAttachmentReference.layout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;

//     VkAttachmentReference depthAttachmentReference;
//     depthAttachmentReference.attachment =
//         (sampleCount > OVR_SAMPLE_COUNT_1 && EXPLICIT_RESOLVE == 0) ? 2 : 1;
//     depthAttachmentReference.layout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;

//     VkAttachmentReference fragmentDensityAttachmentReference;
//     if ((flags & OVR_RENDERPASS_FLAG_INCLUDE_FRAG_DENSITY) != 0) {
//         fragmentDensityAttachmentReference.attachment = sampleMapAttachment;
//         fragmentDensityAttachmentReference.layout =
//             VK_IMAGE_LAYOUT_FRAGMENT_DENSITY_MAP_OPTIMAL_EXT;
//     }

//     VkSubpassDescription subpassDescription;
//     subpassDescription.flags = 0;
//     subpassDescription.pipelineBindPoint = VK_PIPELINE_BIND_POINT_GRAPHICS;
//     subpassDescription.inputAttachmentCount = 0;
//     subpassDescription.pInputAttachments = NULL;
//     subpassDescription.colorAttachmentCount = 1;
//     subpassDescription.pColorAttachments = &colorAttachmentReference;
//     subpassDescription.pResolveAttachments =
//         (sampleCount > OVR_SAMPLE_COUNT_1 && EXPLICIT_RESOLVE == 0) ? &resolveAttachmentReference
//                                                                     : NULL;
//     subpassDescription.pDepthStencilAttachment =
//         (renderPass->internalDepthFormat != VK_FORMAT_UNDEFINED) ? &depthAttachmentReference : NULL;
//     subpassDescription.preserveAttachmentCount = 0;
//     subpassDescription.pPreserveAttachments = NULL;

//     VkRenderPassCreateInfo renderPassCreateInfo;
//     renderPassCreateInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO;
//     renderPassCreateInfo.pNext = NULL;
//     renderPassCreateInfo.flags = 0;
//     renderPassCreateInfo.attachmentCount = attachmentCount;
//     renderPassCreateInfo.pAttachments = attachments;
//     renderPassCreateInfo.subpassCount = 1;
//     renderPassCreateInfo.pSubpasses = &subpassDescription;
//     renderPassCreateInfo.dependencyCount = 0;
//     renderPassCreateInfo.pDependencies = NULL;

//     VkRenderPassMultiviewCreateInfo multiviewCreateInfo;
//     const uint32_t viewMask = 0b00000011;
//     if (isMultiview) {
//         multiviewCreateInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_MULTIVIEW_CREATE_INFO;
//         multiviewCreateInfo.pNext = NULL;
//         multiviewCreateInfo.subpassCount = 1;
//         multiviewCreateInfo.pViewMasks = &viewMask;
//         multiviewCreateInfo.dependencyCount = 0;
//         multiviewCreateInfo.correlationMaskCount = 1;
//         multiviewCreateInfo.pCorrelationMasks = &viewMask;

//         renderPassCreateInfo.pNext = &multiviewCreateInfo;
//     }

//     VkRenderPassFragmentDensityMapCreateInfoEXT fragmentDensityMapCreateInfoEXT;
//     if ((flags & OVR_RENDERPASS_FLAG_INCLUDE_FRAG_DENSITY) != 0) {
//         fragmentDensityMapCreateInfoEXT.sType =
//             VK_STRUCTURE_TYPE_RENDER_PASS_FRAGMENT_DENSITY_MAP_CREATE_INFO_EXT;
//         fragmentDensityMapCreateInfoEXT.fragmentDensityMapAttachment =
//             fragmentDensityAttachmentReference;

//         fragmentDensityMapCreateInfoEXT.pNext = renderPassCreateInfo.pNext;
//         renderPassCreateInfo.pNext = &fragmentDensityMapCreateInfoEXT;
//     }

//     VK(context->device->vkCreateRenderPass(
//         context->device->device, &renderPassCreateInfo, VK_ALLOCATOR, &renderPass->renderPass));

//     return true;
// }
