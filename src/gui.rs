//! Native Win32 GUI: no browser engine, .NET runtime or external GUI DLLs.
use crate::core;
use std::{env, ffi::c_void, mem, os::windows::ffi::OsStrExt, path::PathBuf, ptr, sync::atomic::{AtomicBool, Ordering}, thread};

type Hwnd = *mut c_void;
type Hinstance = *mut c_void;
type Hdrop = *mut c_void;
type Hmenu = *mut c_void;
type Lresult = isize;
type Wparam = usize;
type Lparam = isize;
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_DROPFILES: u32 = 0x0233;
const WM_APP_LOG: u32 = 0x8001;
const WM_APP_DONE: u32 = 0x8002;
const WM_SETFONT: u32 = 0x0030;
const WM_SETTEXT: u32 = 0x000C;
const EM_SETSEL: u32 = 0x00B1;
const EM_REPLACESEL: u32 = 0x00C2;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_VSCROLL: u32 = 0x00200000;
const WS_BORDER: u32 = 0x00800000;
const ES_MULTILINE: u32 = 0x0004;
const ES_AUTOVSCROLL: u32 = 0x0040;
const ES_READONLY: u32 = 0x0800;
const SW_SHOW: i32 = 5;
const MB_OK: u32 = 0;
const MB_ICONINFORMATION: u32 = 0x40;
const MB_ICONWARNING: u32 = 0x30;
const COLOR_WINDOW: usize = 5;
const DEFAULT_GUI_FONT: i32 = 17;
const IDC_ARROW: usize = 32512;

#[repr(C)]
struct WndClassW {
    style: u32, wnd_proc: Option<unsafe extern "system" fn(Hwnd,u32,Wparam,Lparam)->Lresult>,
    cls_extra: i32, wnd_extra: i32, instance: Hinstance, icon: Hinstance,
    cursor: Hinstance, background: Hinstance, menu_name: *const u16, class_name: *const u16,
}
#[repr(C)]
struct Msg { hwnd: Hwnd, message: u32, wparam: Wparam, lparam: Lparam, time: u32, pt_x: i32, pt_y: i32, private: u32 }
#[link(name="user32")]
extern "system" {
    fn MessageBoxW(hwnd:Hwnd,text:*const u16,caption:*const u16,flags:u32)->i32;
    fn RegisterClassW(c: *const WndClassW) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, name: *const u16, style: u32, x:i32,y:i32,w:i32,h:i32,parent:Hwnd,menu:Hmenu,inst:Hinstance,param:*mut c_void)->Hwnd;
    fn DefWindowProcW(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult;
    fn PostQuitMessage(code:i32);
    fn GetMessageW(msg:*mut Msg, hwnd:Hwnd, min:u32,max:u32)->i32;
    fn TranslateMessage(msg:*const Msg)->i32;
    fn DispatchMessageW(msg:*const Msg)->Lresult;
    fn ShowWindow(hwnd:Hwnd,show:i32)->i32;
    fn UpdateWindow(hwnd:Hwnd)->i32;
    fn SendMessageW(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult;
    fn PostMessageW(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->i32;
    fn MoveWindow(hwnd:Hwnd,x:i32,y:i32,w:i32,h:i32,repaint:i32)->i32;
    fn LoadCursorW(inst:Hinstance,id:*const u16)->Hinstance;
    fn LoadIconW(inst:Hinstance,id:*const u16)->Hinstance;
    fn GetClientRect(hwnd:Hwnd,rect:*mut Rect)->i32;
    fn GetSysColorBrush(index:i32)->Hinstance;
}
#[link(name="kernel32")]
extern "system" { fn GetModuleHandleW(name:*const u16)->Hinstance; }
#[link(name="shell32")]
extern "system" { fn DragAcceptFiles(hwnd:Hwnd,accept:i32); fn DragQueryFileW(drop:Hdrop,index:u32,file:*mut u16,length:u32)->u32; fn DragFinish(drop:Hdrop); }
#[link(name="gdi32")]
extern "system" { fn GetStockObject(index:i32)->Hinstance; }
#[repr(C)]
#[derive(Default)]
struct Rect { left:i32,top:i32,right:i32,bottom:i32 }
static BUSY: AtomicBool = AtomicBool::new(false);
static mut LOG: Hwnd = ptr::null_mut();
static mut DROP_LABEL: Hwnd = ptr::null_mut();
static mut STATUS: Hwnd = ptr::null_mut();
static mut LANGUAGE: &str = "en";

fn wide(s:&str)->Vec<u16> { std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect() }
fn language()->&'static str {
    for arg in env::args().skip(1) {
        if let Some(code)=arg.strip_prefix("--lang=") { return core::normalized_lang(code).unwrap_or("en"); }
    }
    core::system_language()
}
fn labels(lang:&str)->(&'static str,&'static str,&'static str,&'static str,&'static str) {
    match lang {
        "es"=>("bzipit — Compresor BZIP2","Arrastra aquí archivos o carpetas","Listo para comprimir","Comprimiendo...","Finalizado"),
        "gl"=>("bzipit — Compresor BZIP2","Arrastra aquí ficheiros ou cartafoles","Listo para comprimir","Comprimindo...","Rematado"),
        "ca"=>("bzipit — Compressor BZIP2","Arrossega aquí fitxers o carpetes","A punt per comprimir","Comprimint...","Finalitzat"),
        "val"=>("bzipit — Compressor BZIP2","Arrossega ací fitxers o carpetes","Preparat per a comprimir","Comprimint...","Finalitzat"),
        "eu"=>("bzipit — BZIP2 konpresorea","Arrastatu fitxategiak edo karpetak hona","Konprimitzeko prest","Konprimitzen...","Amaituta"),
        "fr"=>("bzipit — Compresseur BZIP2","Déposez les fichiers ou dossiers ici","Prêt à compresser","Compression...","Terminé"),
        "pt"=>("bzipit — Compressor BZIP2","Arraste ficheiros ou pastas para aqui","Pronto a comprimir","A comprimir...","Concluído"),
        "de"=>("bzipit — BZIP2-Kompressor","Dateien oder Ordner hier ablegen","Bereit zum Komprimieren","Komprimierung läuft...","Fertig"),
        "it"=>("bzipit — Compressore BZIP2","Trascina qui file o cartelle","Pronto per comprimere","Compressione...","Completato"),
        _=>("bzipit — BZIP2 Compressor","Drop files or folders here","Ready to compress","Compressing...","Finished"),
    }
}
unsafe fn set_text(hwnd:Hwnd,text:&str) { let s=wide(text); SendMessageW(hwnd,WM_SETTEXT,0,s.as_ptr() as isize); }
unsafe fn append_log(text:&str) {
    let s=wide(&format!("{text}\r\n"));
    SendMessageW(LOG,EM_SETSEL,usize::MAX,-1);
    SendMessageW(LOG,EM_REPLACESEL,0,s.as_ptr() as isize);
}
unsafe fn child(parent:Hwnd,class:&str,text:&str,style:u32,x:i32,y:i32,w:i32,h:i32)->Hwnd {
    let hwnd=CreateWindowExW(0,wide(class).as_ptr(),wide(text).as_ptr(),WS_VISIBLE|WS_CHILD|style,x,y,w,h,parent,ptr::null_mut(),GetModuleHandleW(ptr::null()),ptr::null_mut());
    SendMessageW(hwnd,WM_SETFONT,GetStockObject(DEFAULT_GUI_FONT) as usize,1);
    hwnd
}
fn enqueue(hwnd:Hwnd,paths:Vec<PathBuf>) {
    if paths.is_empty() || BUSY.swap(true,Ordering::AcqRel) { return; }
    unsafe { set_text(STATUS,labels(LANGUAGE).3); }
    let window_id=hwnd as usize;
    thread::spawn(move || {
        let stats=core::process_paths(paths,|line| {
            let ptr=Box::into_raw(Box::new(line));
            if unsafe { PostMessageW(window_id as Hwnd,WM_APP_LOG,0,ptr as isize) }==0 { unsafe { drop(Box::from_raw(ptr)); } }
        });
        let boxed=Box::into_raw(Box::new(stats));
        if unsafe { PostMessageW(window_id as Hwnd,WM_APP_DONE,0,boxed as isize) }==0 { unsafe { drop(Box::from_raw(boxed)); } }
    });
}
unsafe extern "system" fn wnd_proc(hwnd:Hwnd,msg:u32,wp:Wparam,lp:Lparam)->Lresult {
    match msg {
        WM_CREATE=> {
            DROP_LABEL=child(hwnd,"STATIC",labels(LANGUAGE).1,WS_BORDER,16,16,650,120);
            LOG=child(hwnd,"EDIT","",WS_BORDER|WS_VSCROLL|ES_MULTILINE|ES_AUTOVSCROLL|ES_READONLY,16,148,650,240);
            STATUS=child(hwnd,"STATIC",labels(LANGUAGE).2,0,16,398,650,26);
            DragAcceptFiles(hwnd,1);
            0
        }
        WM_SIZE=> {
            let mut rect=Rect::default(); GetClientRect(hwnd,&mut rect);
            let w=(rect.right-rect.left-32).max(100); let h=rect.bottom-rect.top;
            MoveWindow(DROP_LABEL,16,16,w,120,1);
            MoveWindow(LOG,16,148,w,(h-198).max(60),1);
            MoveWindow(STATUS,16,(h-40).max(0),w,24,1);
            0
        }
        WM_DROPFILES=> {
            let drop=wp as Hdrop;
            let count=DragQueryFileW(drop,u32::MAX,ptr::null_mut(),0);
            let mut paths=Vec::new();
            for i in 0..count {
                let len=DragQueryFileW(drop,i,ptr::null_mut(),0);
                let mut buf=vec![0u16;len as usize+1];
                if DragQueryFileW(drop,i,buf.as_mut_ptr(),buf.len() as u32)>0 {
                    paths.push(PathBuf::from(std::ffi::OsString::from_wide(&buf[..len as usize])));
                }
            }
            DragFinish(drop);
            if BUSY.load(Ordering::Acquire) { append_log("[BUSY] Wait until the current batch finishes."); }
            else { enqueue(hwnd,paths); }
            0
        }
        WM_APP_LOG=> { let line=Box::from_raw(lp as *mut String); append_log(&line); 0 }
        WM_APP_DONE=> {
            let stats=Box::from_raw(lp as *mut core::Stats);
            append_log(&format!("{}: {} | {}: {} | {}: {}",core::messages(LANGUAGE).compressed,stats.compressed,core::messages(LANGUAGE).skipped,stats.skipped,core::messages(LANGUAGE).errors,stats.failed));
            set_text(STATUS,labels(LANGUAGE).4);
            BUSY.store(false,Ordering::Release);
            0
        }
        WM_DESTROY=> { PostQuitMessage(0); 0 }
        _=>DefWindowProcW(hwnd,msg,wp,lp)
    }
}
pub fn run() {
    // Dragging items onto bzipit.exe supplies their paths as command-line arguments.
    // In that mode, do not create the drop-zone window: process and show one summary.
    let paths: Vec<PathBuf> = env::args_os().skip(1)
        .filter(|arg| !arg.to_string_lossy().starts_with("--lang="))
        .map(PathBuf::from).collect();
    let lang = language();
    if !paths.is_empty() {
        let stats = core::process_paths(paths, |_| {});
        let words = core::messages(lang);
        let summary = format!("{}\n\n{}: {}\n{}: {}\n{}: {}",
            words.summary, words.compressed, stats.compressed,
            words.skipped, stats.skipped, words.errors, stats.failed);
        let caption = wide(labels(lang).0);
        let message = wide(&summary);
        let icon = if stats.failed > 0 { MB_ICONWARNING } else { MB_ICONINFORMATION };
        unsafe { MessageBoxW(ptr::null_mut(), message.as_ptr(), caption.as_ptr(), MB_OK | icon); }
        return;
    }
    unsafe {
        LANGUAGE=lang;
        let inst=GetModuleHandleW(ptr::null());
        let class=wide("bzipitWindow");
        let wc=WndClassW { style:0,wnd_proc:Some(wnd_proc),cls_extra:0,wnd_extra:0,instance:inst,
            icon:LoadIconW(inst,1usize as *const u16),cursor:LoadCursorW(ptr::null_mut(),IDC_ARROW as *const u16),
            background:GetSysColorBrush(COLOR_WINDOW as i32),menu_name:ptr::null(),class_name:class.as_ptr() };
        if RegisterClassW(&wc)==0 { return; }
        let hwnd=CreateWindowExW(0,class.as_ptr(),wide(labels(LANGUAGE).0).as_ptr(),WS_OVERLAPPEDWINDOW|WS_VISIBLE,
            150,150,720,490,ptr::null_mut(),ptr::null_mut(),inst,ptr::null_mut());
        if hwnd.is_null() { return; }
        ShowWindow(hwnd,SW_SHOW); UpdateWindow(hwnd);
        let mut msg:Msg=mem::zeroed();
        while GetMessageW(&mut msg,ptr::null_mut(),0,0)>0 { TranslateMessage(&msg); DispatchMessageW(&msg); }
    }
}
use std::os::windows::ffi::OsStringExt;
