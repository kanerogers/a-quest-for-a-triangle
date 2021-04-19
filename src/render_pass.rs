use ash::{version::DeviceV1_0, vk, Device};

pub struct RenderPass {}

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
