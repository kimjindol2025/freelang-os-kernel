// FreeLang OS Kernel - Phase G: Bare-Metal 구현
// 실제 x86-64 하드웨어에서 부팅되는 OS

#![no_std]
#![no_main]
#![feature(asm_const)]
#![feature(allocator_api)]

extern crate alloc;

use core::fmt::Write;
use core::panic::PanicInfo;

mod serial;
mod vga_buffer;
mod memory;
mod gdt;
mod interrupts;
mod paging;
mod demand_paging;
mod allocator;

use vga_buffer::WRITER;
use memory::PhysicalMemoryManager;

/// Multiboot2 정보 구조체 (간단 버전)
#[repr(C)]
pub struct MultibootInfo {
    size: u32,
    reserved: u32,
}

/// 커널 메인 함수
#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_info: u64) -> ! {
    // VGA 버퍼 초기화
    writeln!(WRITER.lock(), "╔════════════════════════════════════════════════════╗").unwrap();
    writeln!(WRITER.lock(), "║      FreeLang OS Kernel - Phase G Bare-Metal       ║").unwrap();
    writeln!(WRITER.lock(), "║          실제 x86-64 부팅 및 실행                  ║").unwrap();
    writeln!(WRITER.lock(), "╚════════════════════════════════════════════════════╝").unwrap();

    // 시리얼 포트 초기화
    serial::init();
    println!("[SERIAL] Serial port initialized");

    // GDT 초기화
    gdt::init();
    println!("✓ GDT initialized");

    // IDT 초기화
    interrupts::init();
    println!("✓ IDT initialized");

    // 메모리 관리자 초기화
    println!("🔧 Initializing physical memory manager...");
    let _phys_mem = unsafe { PhysicalMemoryManager::new(0x1000) };
    println!("✓ Physical memory manager initialized");

    // Phase 2: Demand Paging 초기화
    println!("\n🔧 Initializing demand paging system...");
    let demand_paging_mgr = demand_paging::DEMAND_PAGING.lock();
    println!("✓ Demand paging manager ready");
    drop(demand_paging_mgr);

    // 힙 할당자 초기화
    println!("🔧 Initializing heap allocator...");
    {
        let alloc = allocator::HEAP_ALLOCATOR.lock();
        println!("✓ Heap allocator initialized");
        println!("  Heap range: 0x200000 ~ 0x20000000 (510 MB)");
        drop(alloc);
    }

    // 인터럽트 활성화
    unsafe { asm!("sti"); }
    println!("✓ Interrupts enabled");

    // 메모리 상태 출력
    println!("\n📊 System Memory Status:");
    {
        let alloc = allocator::HEAP_ALLOCATOR.lock();
        alloc.print_status();
    }

    // 메인 루프
    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║           🚀 커널 부팅 완료 (Phase 2)              ║");
    println!("╠════════════════════════════════════════════════════╣");
    println!("║ ✓ Multiboot2 부트로더                             ║");
    println!("║ ✓ GDT/IDT 초기화                                  ║");
    println!("║ ✓ 타이머 & 키보드 인터럽트                        ║");
    println!("║ ✓ Demand Paging 시스템                           ║");
    println!("║ ✓ 힙 할당자 (First-Fit, Best-Fit)               ║");
    println!("╚════════════════════════════════════════════════════╝");
    println!("\n타이머 인터럽트: 4ms마다 발생");
    println!("Demand Paging: 자동 페이지 할당 준비 완료\n");

    kernel_loop();
}

/// 커널 메인 루프
fn kernel_loop() -> ! {
    let mut tick_count = 0;

    loop {
        tick_count += 1;

        // 1초마다 상태 출력 (timer_ticks = 250 = 1000ms)
        if tick_count % 250 == 0 {
            let seconds = tick_count / 250;
            println!("\n⏱️  Uptime: {}s", seconds);

            // 메모리 상태 출력
            {
                let alloc = allocator::HEAP_ALLOCATOR.lock();
                let available = alloc.available_memory();
                let frag = alloc.fragmentation_ratio();
                println!("  📊 Heap: {} KB free, {:.1}% fragmentation",
                    available / 1024, frag * 100.0);
            }

            // Demand Paging 상태
            {
                let dp = demand_paging::DEMAND_PAGING.lock();
                dp.print_status();
            }
        }

        // 5초마다 상세 정보 출력
        if tick_count % 1250 == 0 && tick_count > 0 {
            println!("\n📋 Detailed System Status:");
            {
                let alloc = allocator::HEAP_ALLOCATOR.lock();
                alloc.print_status();
            }
            println!();
        }

        // 프로세스 스케줄링 (Phase 3에서 구현)
        // scheduler_tick();

        // CPU 대기
        unsafe { asm!("hlt"); }
    }
}

/// Panic 핸들러
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    writeln!(
        WRITER.lock(),
        "🔴 KERNEL PANIC: {}",
        info
    ).unwrap();

    println!("PANIC: {}", info);

    loop {
        unsafe { asm!("hlt"); }
    }
}

// 어셈블리 인라인
use core::arch::asm;
