use ash::{version::InstanceV1_0, vk, Instance};

#[derive(Clone, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn find_queue_families(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        for (i, queue) in unsafe { instance.get_physical_device_queue_family_properties(device) }
            .iter()
            .enumerate()
        {
            let i = i as u32;
            if queue.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i);
            }
            indices.present_family = Some(i);
            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }

    pub fn are_same(&self) -> bool {
        self.is_complete() && self.graphics_family == self.present_family
    }
}
