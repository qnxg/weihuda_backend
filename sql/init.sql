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

INSERT INTO `mini_configs` VALUES ('changelog','[\n  {\n    \"time\": \"2025-12-05\",\n    \"content\": [\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"增加了查看考试成绩具体组成的功能。在成绩查询界面点击某一门课程即可查看该课程的成绩组成。\",\n          \"空教室查询支持时间段多选。\"\n        ]\n      },\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了体测预约查询。\",\n          \"修复了部分用户无法查看课表的问题。\",\n          \"修复了修改密码之后无法绑定的问题。\",\n          \"修复了部分用户无法查看大物实验成绩的问题。\",\n          \"修复了财院校区、德智园区宿舍的电量查询。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-11-10\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了大物实验平台修改密码后无法重新绑定的问题。\",\n          \"修复了部分用户无法查看体测成绩的问题。\"\n        ]\n      },\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"考试安排功能可用。\",\n          \"课表加载失败时，课表页面增加重新加载选项。\"\n        ]\n      },\n      {\n        \"title\": \"调整\",\n        \"details\": [\n          \"对大物实验项目剩余时间的显示做了调整。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-10-12\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了首页今日课程卡片课程时间错误的问题。\",\n          \"修复了添加课程后课表没有立即显示已添加课程的问题。\"\n        ]\n      },\n      {\n        \"title\": \"调整\",\n        \"details\": [\n          \"优化了添加课程页面课程节次选择的交互逻辑。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-10-08\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了校园网流量账单时间错乱的问题。\",\n          \"修复了账号密码错误时没有自动跳转到绑定界面的问题。\",\n          \"修复了体测预约查询。\"\n        ]\n      },\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"增加了大物实验成绩和实验安排的查询功能。\"\n        ]\n      },\n      {\n        \"title\": \"更新调整\",\n        \"details\": [\n          \"体测成绩查询界面增加了一些说明。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-09-26\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了上下课高峰期小程序请求失败的问题。\",\n          \"修复了一些用户的课程成绩无法查询的问题。\"\n        ]\n      },\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"课表详情页增加上课班级和上课人数。\",\n          \"成绩查询页面增加对缓考、辅修等的成绩标识。\"\n        ]\n      },\n      {\n        \"title\": \"调整\",\n        \"details\": [\n          \"优化了校园网流量卡片的显示。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-09-19\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了首页一直在加载的问题。（其实我们三天前就修好了，但是忘点发布了，开发人员在此鞠躬道歉）\",\n          \"修复了课表无法查看的问题。\",\n          \"修复了教务系统相关的请求失败问题。\",\n          \"修复了对望麓桥宿舍、南校区部分宿舍、德智园区部分宿舍的电费查询问题。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-09-09\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了课表总显示第一周的bug。\",\n          \"可信电子凭证的全部课程成绩的第三个数据应为“绩点”，现已更正。\",\n          \"又又修复了课表无法查看的 bug。\"\n        ]\n      },\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"成绩查询界面添加了一些说明文字。\",\n          \"账号绑定界面添加了一些说明文字。\",\n          \"增加了如果密码错误，则会弹出提示，自动解除绑定并跳转到绑定界面的功能。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-09-01\",\n    \"content\": [\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"又修复了课表无法查看的 bug。\",\n          \"修复了课表界面、排名查询界面的滚动问题。\"\n        ]\n      },\n      {\n        \"title\": \"更新调整\",\n        \"details\": [\n          \"更换了小程序设置的图标。\",\n          \"课表临时增加选择 2025-2026 秋季学期。\"\n        ]\n      }\n    ]\n  },\n  {\n    \"time\": \"2025-08-31\",\n    \"content\": [\n      {\n        \"title\": \"视觉优化\",\n        \"details\": [\n          \"首页校园网流量卡片改为显示剩余流量\",\n          \"更改课表界面视觉风格。使用对比度较低的颜色方案。课表上方显示月份+日期，课表左侧显示具体节次及其开始结束时间。\",\n          \"更改成绩查询界面视觉风格。突出分数的显示。对“必修课”、“核心课”的标识进行高亮。\",\n          \"受教务系统请求频率的限制，排名查询界面重做，被迫改为一次只能查询一个成绩和排名。\"\n        ]\n      },\n      {\n        \"title\": \"增加\",\n        \"details\": [\n          \"增加了通过可信电子凭证获取排名的功能。\",\n          \"重新加回了更新日志界面。\"\n        ]\n      },\n      {\n        \"title\": \"修复\",\n        \"details\": [\n          \"修复了校园网流量查询功能。\",\n          \"由于教务系统更新，我们对成绩查询、课表查询、排名查询、空教室查询功能进行修复。\",\n          \"修复课表左右滑动切换周数时跳到第 0 周、无法跳到第 18 周的 bug。\",\n          \"修复课表添加界面图标显示的 bug。\",\n          \"修复了选项卡切换时的 bug。\"\n        ]\n      },\n      {\n        \"title\": \"更新调整\",\n        \"details\": [\n          \"重写了课表渲染的代码，重写了调休相关代码。\",\n          \"对于 2024 级之后的学生，成绩排名中核心课排名选用 2024 版核心课方案。\",\n          \"更新了常见问题界面。\",\n          \"空教室查询的节次选择改为单选。在查询后默认选中查看第一个空教室的详情。\",\n          \"课表的课程详情中增加了课程的备注信息（如有），课程代码，课程性质，学分信息。\",\n          \"成绩查询界面增加了课程代码，所得绩点信息。\",\n          \"问题反馈界面增加先去常见问题查看的提示。\",\n          \"延长了错误信息显示的时间。\"\n        ]\n      },\n      {\n        \"title\": \"移除\",\n        \"details\": [\n          \"由于教务系统更新，机考相关功能、成绩趋势、项目成绩、课程信息被移除。\"\n        ]\n      }\n    ]\n  }\n]'),('classStartDateTable','[\n    [ \"2026-3\", \"2027-07-11\" ],\n    [ \"2026-2\", \"2027-02-21\" ],\n    [ \"2026-1\", \"2026-09-13\" ],\n    [ \"2025-3\", \"2026-07-05\" ],\n    [ \"2025-2\", \"2026-03-01\" ],\n    [ \"2025-1\", \"2025-09-21\" ],\n    [ \"2024-3\", \"2025-06-22\" ],\n    [ \"2024-2\", \"2025-02-16\" ],\n    [ \"2024-1\", \"2024-09-08\" ],\n    [ \"2023-3\", \"2024-06-30\" ],\n    [ \"2023-2\", \"2024-02-25\" ],\n    [ \"2023-1\", \"2023-09-10\" ],\n    [ \"2022-3\", \"2023-06-18\" ],\n    [ \"2022-2\", \"2023-02-12\" ],\n    [ \"2022-1\", \"2022-09-04\" ],\n    [ \"2021-3\", \"2022-06-26\" ],\n    [ \"2021-2\", \"2022-02-20\" ],\n    [ \"2021-1\", \"2021-09-19\" ],\n    [ \"2020-3\", \"2021-07-04\" ],\n    [ \"2020-2\", \"2021-02-28\" ],\n    [ \"2020-1\", \"2020-09-20\" ],\n    [ \"2019-3\", \"2020-07-05\" ],\n    [ \"2019-2\", \"2020-03-01\" ],\n    [ \"2019-1\", \"2019-09-08\" ],\n    [ \"2018-3\", \"2019-06-30\" ],\n    [ \"2018-2\", \"2019-02-24\" ],\n    [ \"2018-1\", \"2018-09-16\" ],\n    [ \"2017-3\", \"2018-07-08\" ],\n    [ \"2017-2\", \"2018-03-04\" ],\n    [ \"2017-1\", \"2017-09-17\" ],\n    [ \"2016-3\", \"2017-06-25\" ],\n    [ \"2016-2\", \"2017-02-19\" ],\n    [ \"2016-1\", \"2016-09-11\" ],\n    [ \"2015-3\", \"2016-08-14\" ],\n    [ \"2015-2\", \"2016-02-28\" ],\n    [ \"2015-1\", \"2015-09-20\" ],\n    [ \"2014-3\", \"2015-08-23\" ],\n    [ \"2014-2\", \"2015-03-08\" ],\n    [ \"2014-1\", \"2014-09-21\" ]\n]'),('faq','[\n  {\n    \"title\": \"登录相关\",\n    \"items\": [\n      {\n        \"q\": \"密码错误/忘记密码\",\n        \"a\": \"请确保自己能够成功登录个人门户：https://pt.hnu.edu.cn/。对于新生，个人门户的初始密码的说明位于随录取通知书一并发放的入学指南中。如忘记密码，请前往个人门户进行密码重置。\"\n      },\n      {\n        \"q\": \"毕业生无法登录\",\n        \"a\": \"毕业生的个人门户账号会被学校回收，无法继续使用微生活小程序。\"\n      }\n    ]\n  },\n  {\n    \"title\": \"课表相关\",\n    \"items\": [\n      {\n        \"q\": \"如何查看课程详情\",\n        \"a\": \"在课表界面点击课程方块即可查看课程详情，会显示一同上课的班级、上课周数以及授课教师。点击外部区域可关闭详情显示。\"\n      },\n      {\n        \"q\": \"课程方块颜色的介绍\",\n        \"a\": \"彩色表示本周要上的课程，灰色表示本周无课的课程，可以参照下面的方法设置不显示非本周课程。课程方块的彩色颜色是随机生成的，如果你有补充的配色方案欢迎联系我们以添加。\"\n      },\n      {\n        \"q\": \"如何添加自定义课程\",\n        \"a\": \"在底部导航栏中选择课程表页面，点击右下角的“+”按钮，填写课程信息后保存即可添加自定义课程。\"\n      },\n      {\n        \"q\": \"如何删除自定义课程\",\n        \"a\": \"在课表界面点击对应课程的方块，之后点击删除按钮即可。\"\n      },\n      {\n        \"q\": \"如何设置仅显示本周课程\",\n        \"a\": \"在“课表”界面点击向下的箭头展开设置界面，启用“仅显示本周课程”即可。\"\n      },\n      {\n        \"q\": \"为何选课成功但微生活课表中不显示\",\n        \"a\": \"有三种可能：1、教务系统出于性能的考虑会将选课结果保存在缓存，并定期同步到课表。在选课成功几分钟到十几分钟后刷新课表就能看到所选课程。2、开启了“仅显示本周课程”功能而所选课程本周不上课，可在课表界面点击向下的箭头展开设置界面，取消该功能。3、该课程是无课表课程，未在系统中安排具体上课时间，无法在课表中显示。\"\n      }\n    ]\n  },\n  {\n    \"title\": \"查询相关\",\n    \"items\": [\n      {\n        \"q\": \"如何刷新查询结果\",\n        \"a\": \"在查询页面下拉可以刷新查询结果。如果结果仍没有更新，可尝试再次下拉刷新，或是点击“我的” -> “清除缓存”\"\n      },\n      {\n        \"q\": \"请求失败，状态码为403\",\n        \"a\": \"在“我的”中点击清除缓存，然后重启小程序即可正常使用。\"\n      },\n      {\n        \"q\": \"提示权限鉴定失败\",\n        \"a\": \"在“我的”中点击清除缓存，然后重启小程序即可正常使用。\"\n      },\n      {\n        \"q\": \"成绩/排名信息加载失败\",\n        \"a\": \"在期末考试周前后，教务系统会有较大访问压力，可能会导致成绩/排名信息加载失败。建议避开高峰期再尝试查询。\"\n      },\n      {\n        \"q\": \"其他请求失败情况\",\n        \"a\": \"请前往学校官方对应的平台查询相关数据，确认一下是否是由于学校平台正在维护导致请求失败。如果并非学校平台问题，请点击“我的” -> “我要反馈”进行反馈，我们将在第一时间修复。\"\n      }\n    ]\n  },\n  {\n    \"title\": \"其他\",\n    \"items\": [\n      {\n        \"q\": \"如何添加微生活图标到桌面\",\n        \"a\": \"点击右上角的三个小点，选择“添加到桌面”，若添加不成功请授予微信“创建桌面快捷方式”权限后重试。（IOS系统不支持该功能）\"\n      },\n      {\n        \"q\": \"首页的卡片排序恢复默认\",\n        \"a\": \"在使用“清除缓存”功能后会删除保存在缓存中的自定义排序结果，重新设置顺序即可。\"\n      },\n      {\n        \"q\": \"如何查看体测单项成绩\",\n        \"a\": \"进入体测查询，点击总得分下的彩色条形图中的任意一种颜色，就会显示对应的单项成绩与占比。\"\n      },\n      {\n        \"q\": \"充值后剩余电量没有变化\",\n        \"a\": \"一般情况下电量会在电费充值几分钟后到账。\"\n      },\n      {\n        \"q\": \"宿舍电量不准/更新不及时\",\n        \"a\": \"微生活小程序对电量数据的获取存在缓存，建议点击 “工具箱” -> “电量查询” ，在电量查询界面下拉刷新以更新电量数据。\"\n      },\n      {\n        \"q\": \"转专业后宿舍信息没有更新\",\n        \"a\": \"宿舍信息来自于学工系统。转专业后学工系统中的宿舍信息不会立即更新，建议联系学校相关部门进行咨询。\"\n      },\n      {\n        \"q\": \"流量查询结果介绍\",\n        \"a\": \"下箭头左边的数字为下载流量，上箭头左边的数字为上传流量。状态显示为未锁定表示校园网账号能够正常使用，流量用超后会被锁定。\"\n      },\n      {\n        \"q\": \"其他问题\",\n        \"a\": \"如果你有其他关于湖大微生活小程序有关的问题，请点击“我的” -> “我要反馈”进行反馈。我们看到后会尽快处理。其他与小程序无关的问题请联系学校的相关部门进行咨询。\"\n      }\n    ]\n  }\n]\n'),('flexTime','[\n    {\n        \"from\": {\n            \"week\": 12,\n            \"day\": 1\n        },\n        \"to\": {\n            \"week\": 11,\n            \"day\": 7\n        },\n        \"desc\": \"4.27补上5.5的课程\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    },\n    {\n        \"from\": null,\n        \"to\": {\n            \"week\": 11,\n            \"day\": 4\n        },\n        \"desc\": \"5.1假期停课\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    },\n    {\n        \"from\": null,\n        \"to\": {\n            \"week\": 11,\n            \"day\": 5\n        },\n        \"desc\": \"5.2假期停课\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    },\n    {\n        \"from\": null,\n        \"to\": {\n            \"week\": 11,\n            \"day\": 6\n        },\n        \"desc\": \"5.3假期停课\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    },\n    {\n        \"from\": null,\n        \"to\": {\n            \"week\": 12,\n            \"day\": 7\n        },\n        \"desc\": \"5.4假期停课\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    },\n    {\n        \"from\": null,\n        \"to\": {\n            \"week\": 12,\n            \"day\": 1\n        },\n        \"desc\": \"5.5假期停课\",\n        \"time\": {\n            \"xn\": 2024,\n            \"xq\": 2\n        }\n    }\n]'),('jifenDesc','<h1 style=\"text-align: center;\"><strong>奖品领取</strong></h1>\n<p style=\"text-align: left;\"><strong>领取时间：</strong></p>\n<p style=\"text-align: left;\"><strong>每周2/6下午14-16时（节假日时间另定）25年寒假（1月12日-2月18日）暂停值班</strong></p>\n<p style=\"text-align: left;\"><strong>领取地点：</strong></p>\n<p style=\"text-align: left;\"><strong>天马学生公寓二区六栋靠近羽毛球场一侧一楼 ——易千网络工作室办公室（铁栅栏门）</strong></p>\n<p style=\"text-align: left;\"><strong>（天马园区后门进去左侧第二小栋育人空间正对面）</strong></p>\n<p style=\"text-align: center;\"><img src=\"https://qnxg.cn/upload-2023/editor-20240323/1733143836132-10.webp\" alt=\"\"\n        data-href=\"\" style=\"width: 236.00px;height: 335.23px;\"></p>\n<p style=\"text-align: center;\"><br></p>\n<h1 style=\"text-align: center;\"><strong>积分获取方式</strong></h1>\n<p style=\"text-align: left;\"><strong>1、 签到：用户前往“每日签到”页面签到，每日可领【5】个积分。</strong></p>\n<p style=\"text-align: left;\"><strong>2、 问卷：完成微生活平台发布的各类调研问卷，每份问卷可获得【40】积分；</strong></p>\n<p style=\"text-align: left;\"><strong>3、 浏览小程序文章：点击浏览“知湖”板块内容，每次可获得【5】积分，每日上限【15】积分；</strong></p>\n<p style=\"text-align: left;\"><strong>4、 投稿：在投稿互动类文章中，于该次投稿截止日期前投稿可以获得【30】积分，优秀投稿内容被采纳额外加【50】积分（投稿被采用）；</strong></p>\n<p style=\"text-align: left;\">\n    <strong>注：参与投稿/投稿内容被采纳需在采用推文发出后十五日内向“湖南大学微生活”公众号内发送投稿截图（或相关信息）、学号领取积分，积分将于七日内分发至账户中；</strong></p>\n<p style=\"text-align: left;\"><strong>5、 问题反馈：向微生活提供建议并采纳【25】积分（个人数据暂不属于该范围内）；</strong></p>\n<p style=\"text-align: left;\"><strong>6、 其他：在节日或活动时期，会举行相关积分激励活动，具体情况以当次活动详情为准，有关活动想法可通过问题反馈渠道提议。</strong><img\n        src=\"https://qnxg.cn/upload-2023/editor-20240323/1733143936123-10.webp\" alt=\"\" data-href=\"\"\n        style=\"width: 325.00px;height: 56.15px;\"></p>\n<h2 style=\"text-align: center;\"><strong>奖品设置</strong></h2>\n<p style=\"text-align: left;\"><strong>积分可用于兑换平台提供的各类奖品，包括虚拟奖品、实物奖品及湖大精美文创等。积分兑换奖品均展示在小程序的【兑换奖品】页面，并不定期更新。</strong><img\n        src=\"https://qnxg.cn/upload-2023/editor-20240323/1733143939880-10.webp\" alt=\"\" data-href=\"\"\n        style=\"width: 293.00px;height: 50.61px;\"></p>\n<h3 style=\"text-align: center;\"><strong>温馨提示：</strong></h3>\n<p style=\"text-align: left;\"><strong>1. 限量奖品不定期更新;</strong></p>\n<p style=\"text-align: left;\"><strong>2. 遵循“先兑先得”原则，权益兑完即止。</strong></p>\n<p style=\"text-align: left;\"><br></p>\n<p> </p>\n<p><br></p>'),('nextVacationDate','2026-01-25'),('zhihuTags','[\n    {\n        \"label\": \"通知\",\n        \"value\": \"通知\"\n    },\n    {\n        \"label\": \"讲座\",\n        \"value\": \"讲座\"\n    },\n    {\n        \"label\": \"原创\",\n        \"value\": \"原创\"\n    },\n    {\n        \"label\": \"活动\",\n        \"value\": \"活动\"\n    },\n    {\n        \"label\": \"微生活指南\",\n        \"value\": \"微生活指南\"\n    }\n]');

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
