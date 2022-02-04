CREATE DATABASE `like-gatherer`;
CREATE TABLE `like-gatherer`.`users` (
    `id` INT UNSIGNED NOT NULL,
    `username` VARCHAR(16) UNIQUE NOT NULL COMMENT 'The username of this user',
    `password` VARCHAR(60) NOT NULL COMMENT 'Password with bcrypt',
    PRIMARY KEY (`id`),
    INDEX(`username`)
) ENGINE = InnoDB;
CREATE TABLE `like-gatherer`.`board` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `creator` INT UNSIGNED NOT NULL COMMENT 'Who created this thing',
    `data` TEXT NOT NULL COMMENT 'What it says',
    PRIMARY KEY (`id`),
    FOREIGN KEY (`creator`) REFERENCES users (`id`)
) ENGINE = InnoDB;
CREATE TABLE `like-gatherer`.`likes` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `board_id` INT UNSIGNED NOT NULL COMMENT 'What thread has been liked',
    `liker` INT UNSIGNED NOT NULL COMMENT 'Who liked this thread',
    PRIMARY KEY (`id`),
    FOREIGN KEY (`liker`) REFERENCES users (`id`),
    FOREIGN KEY (`board_id`) REFERENCES board (`id`),
    UNIQUE KEY `unique_like` (`liker`, `board_id`)
) ENGINE = InnoDB;