use ash::vk;
use parking_lot::RwLockWriteGuard;
use tracing::error;
use std::sync::Arc;
use crate::{core::{CommandPool, CommandPoolBuilder, Queue, SwapchainError, VulkanError, VulkanResult}, render_context::{GraphicsDevice, RenderContext, WindowManager}};


pub struct FrameContext<'a> {
    pub device: &'a GraphicsDevice,
    pub window: &'a RwLockWriteGuard<'a, WindowManager>,
    pub cmd: vk::CommandBuffer,
    pub image_index: u32
}


pub struct RenderLoop {
    pool: CommandPool,
    cmd_buffers: Vec<vk::CommandBuffer>
}

impl RenderLoop {
    pub fn new(ctx: &Arc<RenderContext>) -> Self {
        let pool = CommandPoolBuilder::reset(&ctx.device)
            .build()
            .unwrap();

        let cmd_buffers = pool.allocate_cmd_buffers(
            &ctx.device, 
            vk::CommandBufferLevel::PRIMARY, 
            ctx.frame_in_flight() as u32
        ).unwrap();

        Self {
            pool,
            cmd_buffers
        }
    }

    pub fn draw<F>(&self, ctx: &Arc<RenderContext>, callback: F) -> VulkanResult<()>
    where F: FnOnce(FrameContext<'_>) {

        let device = &ctx.device;
        let mut window_write = ctx.window.write();

        let current_frame = window_write.current_frame;
        let frame_idx = current_frame % device.frame_in_flight;
        let cmd = self.cmd_buffers[frame_idx];
        
        unsafe {

            let sync = &window_write.frame_sync[frame_idx];

            let wait = device.wait_for_fences(&[sync.in_flight_fence.raw], true, u64::MAX);

            if let Err(err) = wait {
                error!("Error wait for fences: {:?}", err);
                return Ok(());
            }

            device
                .reset_fences(&[sync.in_flight_fence.raw])
                .map_err(VulkanError::Unknown)?;

            device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .map_err(VulkanError::Unknown)?;

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device
                .begin_command_buffer(cmd, &begin_info)
                .map_err(VulkanError::Unknown)?;

            let image_index = 
                match window_write.swapchain.loader.acquire_next_image(
                    window_write.swapchain.raw,
                    u64::MAX,
                    sync.image_available.raw,
                    vk::Fence::null(),
                ) {
                    Ok((index, _)) => index,
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        return Err(VulkanError::Swapchain(
                            SwapchainError::SwapchainOutOfDateKhr,
                        ));
                    },
                    Err(e) => return Err(VulkanError::Unknown(e)),
                };


            callback(FrameContext { 
                device, 
                window: &window_write, 
                cmd, 
                image_index
            });
                    
            device
                .end_command_buffer(cmd)
                .map_err(VulkanError::Unknown)?;

            let sync = &window_write.frame_sync[frame_idx];

            let wait_semaphores = [sync.image_available.raw];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [sync.render_finished.raw];

            let binding = [cmd];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&binding)
                .signal_semaphores(&signal_semaphores);

            let binding = ctx.device.queue_pool.get_wait(vk::QueueFlags::GRAPHICS).unwrap();
            let queue = binding.lock().unwrap();
     
            device
                .queue_submit(queue.raw, &[submit_info], sync.in_flight_fence.raw)
                .map_err(VulkanError::Unknown)?;

            let swapchain = [window_write.swapchain.raw];
            let image_indices = [image_index];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchain)
                .image_indices(&image_indices);

            window_write
                .swapchain
                .loader
                .queue_present(queue.raw, &present_info)
                .map_err(VulkanError::Unknown)?;            
        }

        window_write.current_frame += 1;

        Ok(())
    }

    pub fn destroy(&self, ctx: &Arc<RenderContext>) {
        unsafe { 
            ctx.device.destroy_command_pool(self.pool.raw, None) 
        };
    }
}