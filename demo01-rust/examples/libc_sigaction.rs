/// sigaction 函数接受一个 sigaction 结构体作为参数,该结构体定义了信号处理程序的行为。sigaction 结构体包含以下重要字段:
///
/// sa_sigaction: 指向信号处理程序函数的指针。
///
/// sa_flags: 用于配置信号处理程序的行为,如 SA_SIGINFO 标志用于指示处理程序应该接收更多的信号信息。
///
/// sa_mask: 在信号处理程序执行期间,哪些其他信号应该被阻塞。
///
///        | 参数         | 方向 | 作用                       | 是否可为NULL |
///        | :----------- | :--- | :------------------------- | :----------- |
///        | `new_action` | 输入 | 要设置的**新**信号处理行为 | 是           |
///        | `old_action` | 输出 | 返回**旧**的信号处理行为   | 是           |
///
/// ```
///         libc::sigaction(
///             libc::SIGINT,
///             &new_action as *const libc::sigaction,
///             &mut old_action as *mut libc::sigaction,
///         );
/// ```
///

use std::{mem, ptr, thread};
use std::sync::Mutex;
use std::time::Duration;
use libc::{c_int, c_void, sigaction, SIGINT, SIGTERM, SIGQUIT, sighandler_t};
use std::thread::sleep;
use lazy_static::lazy_static;






fn hello() {
    println!("Hello, world!");
    sleep(Duration::from_secs(1));
}

// 所有需要处理的退出信号
const EXIT_SIGNALS: [c_int; 14] = [
    libc::SIGHUP,    // 1 - 终端挂断
    libc::SIGINT,    // 2 - 中断 (Ctrl+C)
    libc::SIGQUIT,   // 3 - 退出 (Ctrl+\)
    libc::SIGILL,    // 4 - 非法指令
    libc::SIGABRT,   // 6 - 异常中止
    libc::SIGFPE,    // 8 - 浮点异常
    libc::SIGSEGV,   // 11 - 段错误
    libc::SIGBUS,    // 7 - 总线错误
    libc::SIGPIPE,   // 13 - 管道破裂
    libc::SIGALRM,   // 14 - 闹钟超时
    libc::SIGTERM,   // 15 - 终止信号
    libc::SIGXCPU,   // 24 - CPU时间超限
    libc::SIGXFSZ,   // 25 - 文件大小超限
    libc::SIGSYS,
    // 注意：SIGKILL (9) 和 SIGSTOP (19) 无法被捕获
];

extern "C" fn handle_all_signals(
    sig: c_int,
    info: *mut libc::siginfo_t,
    context: *mut c_void
) {
    cleanup();
    let old_handlers = OLD_HANDLERS.lock().unwrap();
    for &(old_sig, ref old_sig_action) in old_handlers.iter() {
        if old_sig == sig {
            unsafe {
                let handler = old_sig_action.sa_sigaction;

                // 检查是否是有效的处理器
                if handler != 0 && handler != libc::SIG_DFL as usize && handler != libc::SIG_IGN as sighandler_t {
                    // 调用旧的处理器
                    let handler_func: extern "C" fn(c_int, *mut libc::siginfo_t, *mut c_void) = std::mem::transmute(handler);
                    handler_func(sig, info, context);
                    break;
                } else {
                    break;
                }
            }
        }
    }
    std::process::exit(0);
}

fn cleanup() {
    println!("执行清理工作...");
}

lazy_static! {
    static ref OLD_HANDLERS: Mutex<Vec<(c_int, libc::sigaction)>> = Mutex::new(Vec::new());
}

#[cfg(not(target_os = "windows"))]
fn main() {

    unsafe {
        let sig_action = libc::sigaction {
            sa_sigaction: handle_all_signals as libc::sighandler_t,
            sa_mask: std::mem::zeroed(),
            sa_flags: libc::SA_SIGINFO,
            sa_restorer: None,
        };

        let mut old_action: libc::sigaction = core::mem::zeroed();
        let mut handlers = OLD_HANDLERS.lock().unwrap();
        for &sig in EXIT_SIGNALS.iter() {
            libc::sigaction(sig, &sig_action, &mut old_action as *mut libc::sigaction);
            handlers.push((sig, old_action))
        }
    }

    loop {
        hello()
    }

    thread::sleep(Duration::from_secs(20));
    let signals = [libc::SIGINT, libc::SIGTERM, libc::SIGQUIT, libc::SIGHUP];
}

#[cfg(target_os = "windows")]
fn main() {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::consoleapi::SetConsoleCtrlHandler;
    use winapi::um::wincon::CTRL_C_EVENT;

    extern "system" fn ctrl_c_handler(_: DWORD) -> winapi::ctypes::c_int {
        // Perform cleanup or other actions here
        println!("Caught Ctrl-C signal!");
        // Exit the program
        std::process::exit(0);
        0
    }

    unsafe {
        SetConsoleCtrlHandler(Some(ctrl_c_handler), 1);
    }

    loop {
        hello()
    }
}