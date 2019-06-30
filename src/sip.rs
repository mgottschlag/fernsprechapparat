use pjsua::*;

use std::ffi::{CStr, CString};
use std::fmt;
use std::mem;

pub struct Sip {
    account_id: pjsua_acc_id,
}

impl Sip {
    pub fn new(domain: &str, user: &str, password: &str) -> Result<Sip, Error> {
        let account_id = unsafe {
            let status = pjsua_create();
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                return Err(Error {
                    message: "pjsua_create".to_string(),
                    status,
                });
            }

            // Initialize pjsua.
            let mut config: pjsua_config = mem::uninitialized();
            pjsua_config_default(&mut config);

            config.cb.on_incoming_call = Some(Self::on_incoming_call);
            config.cb.on_call_media_state = Some(Self::on_call_media_state);
            config.cb.on_call_state = Some(Self::on_call_state);
            config.cb.on_reg_state2 = Some(Self::on_reg_state);

            let mut log_config: pjsua_logging_config = mem::uninitialized();
            pjsua_logging_config_default(&mut log_config);
            log_config.console_level = 4;

            let status = pjsua_init(&config, &log_config, std::ptr::null());
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                pjsua_destroy();
                return Err(Error {
                    message: "pjsua_init".to_string(),
                    status,
                });
            }

            // Add UDP transport.
            let mut config: pjsua_transport_config = mem::uninitialized();
            pjsua_transport_config_default(&mut config);
            config.port = 5060;
            let status = pjsua_transport_create(
                pjsip_transport_type_e_PJSIP_TRANSPORT_UDP,
                &config,
                std::ptr::null::<i32>() as *mut _,
            );
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                pjsua_destroy();
                return Err(Error {
                    message: "pjsua_transport_create".to_string(),
                    status,
                });
            }

            // Start pjsua.
            let status = pjsua_start();
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                pjsua_destroy();
                return Err(Error {
                    message: "pjsua_start".to_string(),
                    status,
                });
            }

            let mut snd_devs: [pjmedia_snd_dev_info; 8] = mem::uninitialized();
            let mut snd_dev_count = 8u32;
            let status = pjsua_enum_snd_devs(snd_devs.as_mut_ptr(), &mut snd_dev_count);
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                pjsua_destroy();
                return Err(Error {
                    message: "pjsua_enum_snd_devs".to_string(),
                    status,
                });
            }
            println!("{} sound devices", snd_dev_count);
            for i in 0..snd_dev_count {
                println!(
                    "sound device: {}",
                    CStr::from_ptr(snd_devs[i as usize].name.as_ptr())
                        .to_str()
                        .unwrap()
                );
            }
            //int dev_count;
            //pjmedia_aud_dev_index dev_idx;
            //pj_status_t status;
            //dev_count = pjmedia_aud_dev_count();
            //printf("Got %d audio devices\n", dev_count);
            //for (dev_idx=0; dev_idx<dev_count; ++i) {
            //pjmedia_aud_dev_info info;
            //status = pjmedia_aud_dev_get_info(dev_idx, &info);
            //printf("%d. %s (in=%d, out=%d)\n",
            //dev_idx, info.name,
            //info.input_count, info.output_count);
            //}

            // Register to the SIP server by creating an SIP account.
            let mut config: pjsua_acc_config = mem::uninitialized();
            pjsua_acc_config_default(&mut config);
            let id = CString::new(format!("sip:{}@{}", user, domain)).unwrap();
            config.id = c_str_to_pj_str(&id);
            let reg_uri = CString::new(format!("sip:{}", domain)).unwrap();
            config.reg_uri = c_str_to_pj_str(&reg_uri);
            config.cred_count = 1;
            let domain = CString::new("fritz.box").unwrap();
            config.cred_info[0].realm = c_str_to_pj_str(&domain);
            let scheme = CString::new("digest").unwrap();
            config.cred_info[0].scheme = c_str_to_pj_str(&scheme);
            let user = CString::new(user).unwrap();
            config.cred_info[0].username = c_str_to_pj_str(&user);
            config.cred_info[0].data_type =
                pjsip_cred_data_type_PJSIP_CRED_DATA_PLAIN_PASSWD as i32;
            let password = CString::new(password).unwrap();
            config.cred_info[0].data = c_str_to_pj_str(&password);

            let mut account_id: pjsua_acc_id = mem::uninitialized();
            let status = pjsua_acc_add(&config, pj_constants__PJ_TRUE as i32, &mut account_id);
            if status != pj_constants__PJ_SUCCESS as pj_status_t {
                pjsua_destroy();
                return Err(Error {
                    message: "pjsua_acc_add".to_string(),
                    status,
                });
            }

            account_id
        };
        Ok(Sip { account_id })
    }

    extern "C" fn on_incoming_call(
        _account_id: pjsua_acc_id,
        call_id: pjsua_call_id,
        _rdata: *mut pjsip_rx_data,
    ) {
        unsafe {
            let mut call_info: pjsua_call_info = mem::uninitialized();
            pjsua_call_get_info(call_id, &mut call_info as *mut _);

            let caller = String::from_utf8_lossy(std::slice::from_raw_parts(
                call_info.remote_info.ptr as *const u8,
                call_info.remote_info.slen as usize,
            ));
            println!("Incoming call from {}!", caller);
            // TODO
            //    /* Automatically answer incoming calls with 200/OK */
            pjsua_call_answer(call_id, 200, std::ptr::null(), std::ptr::null());
        }
    }

    extern "C" fn on_call_state(call_id: pjsua_call_id, _e: *mut pjsip_event) {
        unsafe {
            let mut call_info: pjsua_call_info = mem::uninitialized();
            pjsua_call_get_info(call_id, &mut call_info as *mut _);

            let call_state = String::from_utf8_lossy(std::slice::from_raw_parts(
                call_info.state_text.ptr as *const u8,
                call_info.state_text.slen as usize,
            ));
            println!("Call state {}: {}", call_id, call_state);
            // TODO
        }
    }

    extern "C" fn on_call_media_state(call_id: pjsua_call_id) {
        unsafe {
            println!("media state");
            let mut call_info: pjsua_call_info = mem::uninitialized();
            pjsua_call_get_info(call_id, &mut call_info as *mut _);
            if call_info.media_status == pjsua_call_media_status_PJSUA_CALL_MEDIA_ACTIVE {
                // When media is active, connect call to sound device.
                pjsua_conf_connect(call_info.conf_slot, 0);
                pjsua_conf_connect(0, call_info.conf_slot);
            }
            // TODO
        }
    }

    extern "C" fn on_reg_state(acc_id: pjsua_acc_id, info: *mut pjsua_reg_info) {
        unsafe {
            println!("on_reg_state: {}", (*info).renew);
            // TODO
        }
    }
}

impl Drop for Sip {
    fn drop(&mut self) {
        unsafe {
            pjsua_destroy();
        }
    }
}

pub struct Error {
    message: String,
    status: pj_status_t,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            let mut buffer = [0u8; PJ_ERR_MSG_SIZE as usize];
            pj_strerror(
                self.status,
                buffer.as_mut_ptr() as *mut i8,
                PJ_ERR_MSG_SIZE as usize,
            );
            write!(f, "{}: {}", self.message, String::from_utf8_lossy(&buffer))
        }
    }
}

fn c_str_to_pj_str(s: &CStr) -> pj_str_t {
    pj_str_t {
        ptr: s.as_ptr() as *mut _,
        slen: s.to_bytes().len() as i64,
    }
}

fn pj_str_to_string(s: pj_str_t) -> String {
    unsafe {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            s.ptr as *const u8,
            s.slen as usize,
        ))
        .to_string()
    }
}
