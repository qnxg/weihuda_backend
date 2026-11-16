USE weihuda;
SET FOREIGN_KEY_CHECKS = 0;


DROP TABLE IF EXISTS `announcement`;
CREATE TABLE `announcement` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(255) NOT NULL COMMENT '公告标题',
  `content` varchar(255) NOT NULL COMMENT '公告内容',
  `url` varchar(255) DEFAULT NULL COMMENT '小程序内的跳转路径',
  `createdAt` datetime NOT NULL COMMENT '创建时间',
  `updatedAt` datetime NOT NULL COMMENT '修改时间',
  `deletedAt` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `fast_reply`;
CREATE TABLE `fast_reply` (
  `id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `content` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

DROP TABLE IF EXISTS `feedback_msg`;
CREATE TABLE `feedback_msg` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `typ` varchar(255) NOT NULL COMMENT '回复类型',
  `msg` varchar(255) DEFAULT NULL COMMENT '回复内容',
  `stuId` varchar(255) NOT NULL COMMENT '发起这个回复的学号',
  `feedbackId` int unsigned NOT NULL,
  `createdAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `feedbackId` (`feedbackId`),
  CONSTRAINT `feedback_msg_ibfk_2` FOREIGN KEY (`feedbackId`) REFERENCES `feedbacks` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `feedbacks`;
CREATE TABLE `feedbacks` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `contact` varchar(255) DEFAULT NULL COMMENT '联系方式',
  `desc` varchar(255) NOT NULL COMMENT '描述',
  `imgUrl` varchar(255) DEFAULT NULL COMMENT '图片地址',
  `stuId` varchar(255) DEFAULT NULL COMMENT '学号',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `status` int unsigned NOT NULL DEFAULT '0' COMMENT '状态',
  `qqbot_msg_id` int DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `jifen_exchange`;
CREATE TABLE `jifen_exchange` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `stuId` varchar(255) NOT NULL COMMENT '学号',
  `goodsId` int unsigned NOT NULL COMMENT '奖品id',
  `status` int unsigned NOT NULL COMMENT '状态',
  `receiveTime` datetime DEFAULT NULL COMMENT '收货时间',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `stuId` (`stuId`),
  CONSTRAINT `jifen_exchange_ibfk_1` FOREIGN KEY (`stuId`) REFERENCES `mini_bind` (`stuId`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `jifen_goods`;
CREATE TABLE `jifen_goods` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `name` varchar(255) NOT NULL COMMENT '名称',
  `cover` varchar(255) NOT NULL COMMENT '图片',
  `count` int unsigned NOT NULL COMMENT '数量',
  `price` int NOT NULL COMMENT '价格',
  `description` varchar(255) DEFAULT NULL COMMENT '描述',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  `enabled` int unsigned NOT NULL DEFAULT '0' COMMENT '启用',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `jifen_records`;
CREATE TABLE `jifen_records` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `key` varchar(255) NOT NULL COMMENT 'key',
  `param` varchar(255) NOT NULL COMMENT '参数',
  `stuId` varchar(255) NOT NULL COMMENT '学号',
  `desc` varchar(255) NOT NULL COMMENT '描述',
  `jifen` int NOT NULL COMMENT '积分',
  `createdAt` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `stuId` (`stuId`),
  CONSTRAINT `jifen_records_ibfk_1` FOREIGN KEY (`stuId`) REFERENCES `mini_bind` (`stuId`) ON DELETE CASCADE
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `jifen_rules`;
CREATE TABLE `jifen_rules` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `key` varchar(255) NOT NULL COMMENT 'key',
  `name` varchar(255) NOT NULL COMMENT '名称',
  `jifen` int NOT NULL COMMENT '积分',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  `cycle` int unsigned NOT NULL COMMENT '周期（天）',
  `maxCount` int unsigned NOT NULL COMMENT '奖励次数上限',
  `isShow` int unsigned NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

INSERT INTO `jifen_rules` VALUES (1,'qiandao','签到',5,'2026-02-15 11:29:52','2026-02-15 11:29:53',NULL,1,1,1),(2,'yuedu','阅读文章（日上限15）',5,'2024-03-23 12:25:22','2025-09-20 07:56:39',NULL,1,3,1);

DROP TABLE IF EXISTS `kv_cache`;
CREATE TABLE `kv_cache` (
  `key` varchar(255) NOT NULL,
  `value` text NOT NULL,
  `update_at` datetime NOT NULL,
  PRIMARY KEY (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `message_lefts`;
CREATE TABLE `message_lefts` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `desc` varchar(255) NOT NULL COMMENT '留言内容',
  `stuId` varchar(255) NOT NULL COMMENT '学号',
  `email` varchar(255) DEFAULT NULL,
  `isAgree` int unsigned NOT NULL COMMENT '是否公开',
  `sendTime` datetime NOT NULL COMMENT '发送时间',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  `isSend` int unsigned DEFAULT NULL COMMENT '是否发送',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `mini_bind`;
CREATE TABLE `mini_bind` (
  `stuId` varchar(255) NOT NULL COMMENT '学号',
  `openid` varchar(255) DEFAULT NULL COMMENT '微信ID',
  `password` varchar(255) NOT NULL COMMENT '个人门户密码',
  `labPass` varchar(255) DEFAULT NULL,
  `room` varchar(255) DEFAULT NULL,
  `jifen` int NOT NULL DEFAULT '0',
  `settings` json DEFAULT NULL,
  `createdAt` datetime NOT NULL COMMENT '创建时间',
  `updatedAt` datetime NOT NULL COMMENT '修改时间',
  `qqOpenid` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`stuId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `mini_configs`;
CREATE TABLE `mini_configs` (
  `key` varchar(255) NOT NULL,
  `value` longtext NOT NULL COMMENT 'value',
  PRIMARY KEY (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

INSERT INTO `mini_configs`
VALUES
    ('classStartDateTable','[
    [ "2026-3", "2027-07-11" ],
    [ "2026-2", "2027-02-21" ],
    [ "2026-1", "2026-09-13" ],
    [ "2025-3", "2026-07-05" ],
    [ "2025-2", "2026-03-01" ],
    [ "2025-1", "2025-09-21" ],
    [ "2024-3", "2025-06-22" ],
    [ "2024-2", "2025-02-16" ],
    [ "2024-1", "2024-09-08" ],
    [ "2023-3", "2024-06-30" ],
    [ "2023-2", "2024-02-25" ],
    [ "2023-1", "2023-09-10" ],
    [ "2022-3", "2023-06-18" ],
    [ "2022-2", "2023-02-12" ],
    [ "2022-1", "2022-09-04" ],
    [ "2021-3", "2022-06-26" ],
    [ "2021-2", "2022-02-20" ],
    [ "2021-1", "2021-09-19" ],
    [ "2020-3", "2021-07-04" ],
    [ "2020-2", "2021-02-28" ],
    [ "2020-1", "2020-09-20" ],
    [ "2019-3", "2020-07-05" ],
    [ "2019-2", "2020-03-01" ],
    [ "2019-1", "2019-09-08" ],
    [ "2018-3", "2019-06-30" ],
    [ "2018-2", "2019-02-24" ],
    [ "2018-1", "2018-09-16" ],
    [ "2017-3", "2018-07-08" ],
    [ "2017-2", "2018-03-04" ],
    [ "2017-1", "2017-09-17" ],
    [ "2016-3", "2017-06-25" ],
    [ "2016-2", "2017-02-19" ],
    [ "2016-1", "2016-09-11" ],
    [ "2015-3", "2016-08-14" ],
    [ "2015-2", "2016-02-28" ],
    [ "2015-1", "2015-09-20" ],
    [ "2014-3", "2015-08-23" ],
    [ "2014-2", "2015-03-08" ],
    [ "2014-1", "2014-09-21" ]
    ]'),
    ('flexTime','[
    {
        "from": {
            "week": 12,
            "day": 1
        },
        "to": {
            "week": 11,
            "day": 7
        },
        "desc": "4.27补上5.5的课程",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    },
    {
        "from": null,
        "to": {
            "week": 11,
            "day": 4
        },
        "desc": "5.1假期停课",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    },
    {
        "from": null,
        "to": {
            "week": 11,
            "day": 5
        },
        "desc": "5.2假期停课",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    },
    {
        "from": null,
        "to": {
            "week": 11,
            "day": 6
        },
        "desc": "5.3假期停课",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    },
    {
        "from": null,
        "to": {
            "week": 12,
            "day": 7
        },
        "desc": "5.4假期停课",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    },
    {
        "from": null,
        "to": {
            "week": 12,
            "day": 1
        },
        "desc": "5.5假期停课",
        "time": {
            "xn": 2024,
            "xq": 2
        }
    }
]'),
    ('jifenDesc','<h1 style="text-align: center;"><strong>奖品领取</strong></h1>
<p style="text-align: left;"><strong>领取时间：</strong></p>
<p style="text-align: left;"><strong>每周2/6下午14-16时（节假日时间另定）25年寒假（1月12日-2月18日）暂停值班</strong></p>
<p style="text-align: left;"><strong>领取地点：</strong></p>
<p style="text-align: left;"><strong>天马学生公寓二区六栋靠近羽毛球场一侧一楼 ——易千网络工作室办公室（铁栅栏门）</strong></p>
<p style="text-align: left;"><strong>（天马园区后门进去左侧第二小栋育人空间正对面）</strong></p>
<p style="text-align: center;"><img src="https://qnxg.cn/upload-2023/editor-20240323/1733143836132-10.webp" alt=""
        data-href="" style="width: 236.00px;height: 335.23px;"></p>
<p style="text-align: center;"><br></p>
<h1 style="text-align: center;"><strong>积分获取方式</strong></h1>
<p style="text-align: left;"><strong>1、 签到：用户前往“每日签到”页面签到，每日可领【5】个积分。</strong></p>
<p style="text-align: left;"><strong>2、 问卷：完成微生活平台发布的各类调研问卷，每份问卷可获得【40】积分；</strong></p>
<p style="text-align: left;"><strong>3、 浏览小程序文章：点击浏览“知湖”板块内容，每次可获得【5】积分，每日上限【15】积分；</strong></p>
<p style="text-align: left;"><strong>4、 投稿：在投稿互动类文章中，于该次投稿截止日期前投稿可以获得【30】积分，优秀投稿内容被采纳额外加【50】积分（投稿被采用）；</strong></p>
<p style="text-align: left;">
    <strong>注：参与投稿/投稿内容被采纳需在采用推文发出后十五日内向“湖南大学微生活”公众号内发送投稿截图（或相关信息）、学号领取积分，积分将于七日内分发至账户中；</strong></p>
<p style="text-align: left;"><strong>5、 问题反馈：向微生活提供建议并采纳【25】积分（个人数据暂不属于该范围内）；</strong></p>
<p style="text-align: left;"><strong>6、 其他：在节日或活动时期，会举行相关积分激励活动，具体情况以当次活动详情为准，有关活动想法可通过问题反馈渠道提议。</strong><img
        src="https://qnxg.cn/upload-2023/editor-20240323/1733143936123-10.webp" alt="" data-href=""
        style="width: 325.00px;height: 56.15px;"></p>
<h2 style="text-align: center;"><strong>奖品设置</strong></h2>
<p style="text-align: left;"><strong>积分可用于兑换平台提供的各类奖品，包括虚拟奖品、实物奖品及湖大精美文创等。积分兑换奖品均展示在小程序的【兑换奖品】页面，并不定期更新。</strong><img
        src="https://qnxg.cn/upload-2023/editor-20240323/1733143939880-10.webp" alt="" data-href=""
        style="width: 293.00px;height: 50.61px;"></p>
<h3 style="text-align: center;"><strong>温馨提示：</strong></h3>
<p style="text-align: left;"><strong>1. 限量奖品不定期更新;</strong></p>
<p style="text-align: left;"><strong>2. 遵循“先兑先得”原则，权益兑完即止。</strong></p>
<p style="text-align: left;"><br></p>
<p> </p>
<p><br></p>'),
    ('nextVacationDate','2026-01-25'),
    ('zhihuTags','[
    {
        "label": "通知",
        "value": "通知"
    },
    {
        "label": "讲座",
        "value": "讲座"
    },
    {
        "label": "原创",
        "value": "原创"
    },
    {
        "label": "活动",
        "value": "活动"
    },
    {
        "label": "微生活指南",
        "value": "微生活指南"
    }
]');


DROP TABLE IF EXISTS `mini_course`;
CREATE TABLE `mini_course` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `stuId` varchar(255) NOT NULL,
  `classname` char(100) NOT NULL COMMENT '课程名称',
  `location` char(50) DEFAULT NULL COMMENT '上课地点',
  `teachers` char(100) DEFAULT NULL COMMENT '授课教师',
  `week` char(100) NOT NULL,
  `day` char(100) NOT NULL,
  `section` char(50) NOT NULL,
  `xn` int unsigned NOT NULL COMMENT '学年',
  `xq` int unsigned DEFAULT NULL COMMENT '学期',
  `createdAt` datetime NOT NULL COMMENT '创建时间',
  `updatedAt` datetime NOT NULL COMMENT '修改时间',
  `deletedAt` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `stuId` (`stuId`),
  CONSTRAINT `mini_course_ibfk_1` FOREIGN KEY (`stuId`) REFERENCES `mini_bind` (`stuId`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `mini_exam_num`;
CREATE TABLE `mini_exam_num` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `examNum` char(100) NOT NULL COMMENT '准考证号',
  `examName` char(100) NOT NULL COMMENT '考试名称',
  `examDate` char(100) NOT NULL COMMENT '考试日期',
  `stuId` varchar(255) NOT NULL,
  `createdAt` datetime NOT NULL COMMENT '创建时间',
  `updatedAt` datetime NOT NULL COMMENT '修改时间',
  `deletedAt` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `stuId` (`stuId`),
  CONSTRAINT `mini_exam_num_ibfk_1` FOREIGN KEY (`stuId`) REFERENCES `mini_bind` (`stuId`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `notices`;
CREATE TABLE `notices` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `content` varchar(255) NOT NULL COMMENT '内容',
  `stuId` varchar(255) NOT NULL COMMENT '收件人',
  `isShow` int unsigned NOT NULL COMMENT '是否完整显示在首页',
  `status` int unsigned NOT NULL COMMENT '状态，0 为未读，1 为已读，已读的通知就不再到首页了',
  `url` varchar(255) DEFAULT NULL,
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

DROP TABLE IF EXISTS `zhihus`;
CREATE TABLE `zhihus` (
  `id` int unsigned NOT NULL AUTO_INCREMENT COMMENT 'id',
  `title` varchar(255) NOT NULL COMMENT '标题',
  `content` longtext NOT NULL COMMENT '内容',
  `tags` varchar(255) NOT NULL COMMENT '标签',
  `cover` varchar(255) DEFAULT NULL COMMENT '封面',
  `status` int unsigned NOT NULL,
  `stuId` varchar(255) NOT NULL COMMENT '发布者的学号',
  `createdAt` datetime NOT NULL,
  `updatedAt` datetime NOT NULL,
  `deletedAt` datetime DEFAULT NULL,
  `top` int unsigned NOT NULL,
  `typ` varchar(255) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

SET FOREIGN_KEY_CHECKS = 1;
