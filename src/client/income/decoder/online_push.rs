use bytes::{Buf, Bytes};
use crate::client::income::decoder::online_push::OnlinePushTrans::{MemberKicked, MemberLeave, MemberPermissionChanged};
use crate::client::outcome::PbToBytes;
use crate::client::structs::{GroupMemberPermission};
use crate::pb::msg;
use crate::pb::msg::{PushMessagePacket, TransMsgInfo};

pub enum OnlinePushTrans {
    MemberLeave {
        msg_uid: i64,
        // 和group_code不一样
        group_uin: i64,
        member_uin: i64,
    },
    MemberKicked {
        msg_uid: i64,
        // 和group_code不一样
        group_uin: i64,
        member_uin: i64,
        operator_uin: i64,
    },
    MemberPermissionChanged {
        msg_uid: i64,
        // 和group_code不一样
        group_uin: i64,
        member_uin: i64,
        new_permission: GroupMemberPermission,
    },
}

// TODO 还没测试
pub fn decode_online_push_trans_packet(payload: &[u8]) -> Option<OnlinePushTrans> {
    let trans_msg_info = TransMsgInfo::from_bytes(payload);
    if trans_msg_info.is_err() {
        return None;
    }
    let info = trans_msg_info.unwrap();
    let msg_uid = info.msg_uid.unwrap_or(0);
    let group_uin = info.from_uin?;
    let mut data = Bytes::from(info.msg_data.unwrap());
    // 去重暂时不做
    match info.msg_type {
        Some(34) => {
            data.get_i32();
            data.get_u8();
            let target = data.get_i32() as i64;
            let typ = data.get_u8() as i32;
            let operator = data.get_i32() as i64;
            match typ {
                0x02 | 0x82 => {
                    return Some(MemberLeave {
                        msg_uid,
                        group_uin,
                        member_uin: target,
                    });
                }
                0x03 | 0x83 => {
                    return Some(MemberKicked {
                        msg_uid,
                        group_uin,
                        member_uin: target,
                        operator_uin: operator,
                    });
                }
                _ => {}
            }
        }
        Some(44) => {
            data.advance(5);
            let var4 = data.get_u8() as i32;
            let mut var5 = 0 as i64;
            let target = data.get_i32() as i64;
            if var4 != 0 && var4 != 1 {
                var5 = data.get_i32() as i64;
            }
            if var5 == 0 && data.len() == 1 {
                let new_permission = if data.get_u8() == 1 { GroupMemberPermission::Administrator } else { GroupMemberPermission::Member };
                return Some(MemberPermissionChanged {
                    msg_uid,
                    group_uin,
                    member_uin: target,
                    new_permission,
                });
            }
        }
        _ => {}
    }
    None
}

#[derive(Debug)]
pub struct GroupMessagePart {
    seq: i32,
    rand: i32,
    group_code: i64,
    from_uin: i64,
    elems: Vec<msg::Elem>,
    time: i32,
    // 语音消息
    ptt: Option<msg::Ptt>,

    // 整个message有多少个part，大于elem.len()时，应等待下一个片段到达后合并
    pkg_num: i32,
    // 分片的第几段
    pkg_index: i32,
    // 分片id，相同id的应该合并，且根据pkg_index排序
    div_seq: i32,
}

// 解析群消息分片 TODO 长消息需要合并
pub fn decode_group_message_packet(payload: &[u8]) -> Option<GroupMessagePart> {
    let pkt = PushMessagePacket::from_bytes(payload);
    if pkt.is_err() {
        return None;
    }
    let message = pkt.unwrap().message.unwrap();

    return Some(GroupMessagePart {
        seq: message.head.as_ref().unwrap().msg_seq.unwrap(),
        rand: message.body.as_ref().unwrap().rich_text.as_ref().unwrap().attr.as_ref().unwrap().random.unwrap(),
        group_code: message.head.as_ref().unwrap().group_info.as_ref().unwrap().group_code.unwrap(),
        from_uin: message.head.as_ref().unwrap().from_uin.unwrap(),
        elems: message.body.as_ref().unwrap().rich_text.as_ref().unwrap().elems.clone(),
        time: message.head.as_ref().unwrap().msg_time.unwrap(),
        pkg_num: message.content.as_ref().unwrap().pkg_num.unwrap(),
        pkg_index: message.content.as_ref().unwrap().pkg_index.unwrap(),
        div_seq: message.content.as_ref().unwrap().div_seq.unwrap(),
        ptt: message.body.as_ref().unwrap().rich_text.as_ref().unwrap().ptt.clone(),
    });
}