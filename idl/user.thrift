namespace rs weihuda.rpc

enum Gender {
    MALE = 1,
    FEMALE = 2,
}

enum Level {
    UNDERGRADUATE = 1,
    POSTGRADUATE = 2,
    DOCTORAL = 3,
}

struct Dormitory {
    1: optional string park,
    2: optional string build,
    3: required string room,
    4: required string raw_dormitory,
}

struct PersonalInfo {
    1: required string name,
    2: required i16 enter_year,
    3: optional i16 xz,
    4: required string stu_id,
    5: required Gender gender,
    6: required Level level,
    7: required string academy,
    8: required string major,
    9: required string class,
    10: optional Dormitory dormitory,
    11: optional string politic,
    12: optional string race,
    13: optional string hometown,
    14: optional string phone,
    15: optional string wechat,
    16: optional string qq,
    17: optional string email,
}

exception RpcError {
    1: required string message,
}

service UserService {
    PersonalInfo get_user_info(1: string jwt)
        throws (1: RpcError error),
}
