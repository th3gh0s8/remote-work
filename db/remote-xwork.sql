-- phpMyAdmin SQL Dump
-- version 5.2.1
-- https://www.phpmyadmin.net/
--
-- Host: localhost
-- Generation Time: Dec 19, 2025 at 12:24 PM
-- Server version: 10.4.32-MariaDB
-- PHP Version: 8.2.12

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `remote-xwork`
--

-- --------------------------------------------------------

--
-- Table structure for table `salesrep`
--

CREATE TABLE `salesrep` (
  `ID` int(11) NOT NULL,
  `br_id` int(11) NOT NULL DEFAULT 1,
  `RepID` varchar(100) DEFAULT NULL,
  `Name` varchar(500) DEFAULT NULL,
  `fullName` varchar(100) NOT NULL,
  `gender` varchar(6) NOT NULL,
  `mobile` varchar(11) NOT NULL,
  `emailAddress` varchar(100) NOT NULL,
  `address` text NOT NULL,
  `nic` varchar(100) NOT NULL,
  `employee_dob` date DEFAULT NULL,
  `wish_viewed_users` varchar(3000) NOT NULL,
  `imei` bigint(50) NOT NULL,
  `repMail` text NOT NULL,
  `join_date` date NOT NULL,
  `salary` double NOT NULL,
  `departmnt` int(11) NOT NULL,
  `Percentage` varchar(100) DEFAULT NULL,
  `Type` varchar(100) DEFAULT NULL,
  `loginCode` varchar(22) NOT NULL,
  `username` varchar(22) NOT NULL,
  `recordDate` date NOT NULL,
  `recordTime` time NOT NULL,
  `userID` int(11) NOT NULL,
  `Actives` varchar(20) NOT NULL DEFAULT 'YES',
  `rep_level` varchar(50) NOT NULL,
  `left_date` date NOT NULL,
  `sales_target` varchar(30) NOT NULL DEFAULT 'NO',
  `payroll_active` varchar(10) NOT NULL DEFAULT 'YES',
  `is_salesmen` int(11) NOT NULL,
  `assigned_reptbs` varchar(100) NOT NULL,
  `cloud` varchar(30) NOT NULL DEFAULT 'Up',
  `mysql_db` varchar(30) DEFAULT NULL,
  `backup` int(11) NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=latin1 COLLATE=latin1_swedish_ci;

--
-- Dumping data for table `salesrep`
--

INSERT INTO `salesrep` (`ID`, `br_id`, `RepID`, `Name`, `fullName`, `gender`, `mobile`, `emailAddress`, `address`, `nic`, `employee_dob`, `wish_viewed_users`, `imei`, `repMail`, `join_date`, `salary`, `departmnt`, `Percentage`, `Type`, `loginCode`, `username`, `recordDate`, `recordTime`, `userID`, `Actives`, `rep_level`, `left_date`, `sales_target`, `payroll_active`, `is_salesmen`, `assigned_reptbs`, `cloud`, `mysql_db`, `backup`) VALUES
(1, 1, '5', 'Amaan', '', 'Male', '0762123334', '', '', '..', '2000-10-04', '102-2025,69-2025,56-2025,25-2025,42-2025,29-2025,90-2025,3-2025', 0, 'amaanashraff@powersoftt.com', '0000-00-00', 0, 0, '10', NULL, '', '', '2025-11-14', '03:34:19', 0, 'YES', 'Servicemen', '0000-00-00', 'NO', 'NO', 1, 'null', 'NEW', NULL, 0),
(2, 1, '002', 'Anoch', '', 'Male', '0154541', '', ' ', '554545454', NULL, '', 0, '', '2019-07-09', 25000, 0, '2', NULL, '123', 'Anoch', '2025-11-27', '10:30:09', 0, 'YES', 'Servicemen', '0000-00-00', 'YES', 'YES', 1, 'null', 'NEW', NULL, 0),
(3, 1, '003', 'Infaz', '', 'Male', '', '', ' ', '52125', NULL, '', 0, '', '2019-07-15', 25000, 23, '5', NULL, '', '', '2024-10-09', '07:30:04', 0, 'NO', 'Servicemen', '0000-00-00', 'YES', 'YES', 1, '', 'NEW', NULL, 0),
(4, 1, '004', 'Jeza', '', 'Male', '0555555555', '', '', '.', NULL, '', 0, '', '2019-07-23', 15000, 14, '1', NULL, '666787', '', '2025-01-07', '01:29:36', 0, 'NO', 'null', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(5, 1, '005', 'Ayub', '', 'Male', '', '', ' ', '12', NULL, '', 0, '', '2019-07-08', 0, 0, '', NULL, '', '', '2025-09-25', '05:12:36', 0, 'YES', 'null', '0000-00-00', 'NO', 'NO', 1, 'null', 'NEW', NULL, 0),
(6, 1, '006', 'Omar ', '', 'Male', '', '', ' ', '20087239832v', NULL, '', 0, '', '2019-07-08', 10000, 0, '', NULL, '', '', '2024-06-18', '10:11:46', 0, 'YES', 'Servicemen', '0000-00-00', 'NO', 'NO', 0, '', 'NEW', NULL, 0),
(7, 1, '007', 'Rifky', '', 'Male', '', '', ' ', '12', NULL, '', 0, '', '0000-00-00', 0, 0, '', NULL, '', '', '2024-06-18', '10:10:29', 0, 'YES', 'null', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(8, 1, '008', 'new', '', 'undefi', '', '', ' ', '123', NULL, '', 0, '', '0000-00-00', 0, 0, '', NULL, '', '', '2020-12-31', '04:18:06', 0, 'YES', 'null', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(9, 1, '009', 'test', '', 'Male', '', '', ' ', '34576890', NULL, '', 0, '', '0000-00-00', 1500, 0, '', NULL, '', '', '2023-12-14', '09:57:18', 0, 'YES', 'null', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(10, 1, '011', 'Yoosuf', '', '', '077123456', '', ' ', '', NULL, '', 0, '', '2019-07-01', 50000, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(11, 1, 'stf1', 'sdfdsf', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-07-12', 10000, 0, '', NULL, '', '', '2019-07-12', '02:47:09', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(12, 1, '012', 'Payroll', '', 'Male', '', '', '', 'undefined', NULL, '', 123, '', '2019-07-12', 0, 23, '', NULL, '', '', '2020-01-08', '09:35:16', 20, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(13, 1, '013', 'Jihas', '', 'Male', '', '', ' ', '', NULL, '', 0, '', '2018-10-02', 25000, 0, '', NULL, '', '', '2019-07-12', '04:52:34', 134, 'YES', 'Level 1', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(14, 1, '014', 'Mobile', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-07-17', 0, 0, '', NULL, '', '', '2019-07-17', '10:13:56', 20, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(15, 1, '015', 'Rifaz', '', '', '', '', ' ', '', NULL, '', 0, '', '2019-07-01', 25000, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(16, 1, '016', 'Facelogin', '', 'Male', '', '', '', 'undefined', NULL, '', 12, '', '2019-07-17', 0, 0, '', NULL, '', '', '2019-07-17', '01:15:57', 20, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(17, 1, '017', 'Soigner', '', 'Male', '', '', '', 'undefined', NULL, '', 123, '', '2019-07-19', 0, 0, '', NULL, '', '', '2019-07-19', '06:35:30', 20, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(18, 4, '0001', 'Admin', '', 'undefi', '', '', '', '00', NULL, '', 0, '', '0000-00-00', 0, 0, '', NULL, '', '', '2019-11-13', '12:06:55', 0, 'YES', 'null', '0000-00-00', 'YES', 'YES', 1, '', 'NEW', NULL, 0),
(19, 1, '4823', 'umar', '', '', '', '', ' ', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(20, 1, '4353', 'Shimar', '', '', '', '', ' ', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(21, 1, '018', 'saleem', '', '', '', '', ' ', '', NULL, '', 0, '', '0000-00-00', 0, 19, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(22, 5, '0001', 'Admin', '', '', '', '', '', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'YES', 1, '', 'Up', NULL, 0),
(23, 5, 'Fuel-001', 'Fuel Demo', '', 'Male', '', '', '', 'undefined', NULL, '', 1, '', '2019-08-09', 0, 0, '', NULL, '', '', '2019-08-09', '10:12:37', 146, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(24, 5, '258984845', 'Ajantha', '', 'Male', '', '', '', 'undefined', NULL, '', 125, '', '2019-08-09', 0, 0, '', NULL, '', '', '2019-08-09', '04:11:52', 147, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(25, 1, '019', 'test', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-08-15', 20000, 0, '', NULL, '', '', '2019-08-15', '03:20:37', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(26, 4, 'Pos-002', 'Staff 1', '', 'Male', '', '', '', 'undefined', NULL, '', 11, '', '2019-08-29', 0, 0, '', NULL, '', '', '2019-08-29', '07:50:09', 144, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(27, 1, '109239', 'Omar', '', 'Male', '', '', 'Katugasthota', 'undefined', NULL, '', 0, '', '2019-09-09', 0, 0, '', NULL, '', '', '2019-09-09', '09:46:49', 127, 'YES', 'Level 1', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(28, 1, '010', 'test fami', '', '', '', '', '', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 1, '', 'Up', NULL, 0),
(29, 1, 'far', 'FARSHAN', '', 'Male', '', '', '', '87452', NULL, '', 0, '', '2019-11-02', 0, 23, '20', NULL, '1234', '', '2025-10-06', '11:40:46', 477, 'YES', 'Level 1', '0000-00-00', 'NO', 'NO', 1, 'null', 'NEW', NULL, 0),
(30, 4, 'POS-003', 'Anas', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-11-13', 0, 0, '', NULL, '', '', '2019-11-13', '11:37:48', 144, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(31, 4, 'POS-004', 'Raheem', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-11-18', 0, 0, '', NULL, '', '', '2019-11-18', '03:28:38', 144, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(33, 4, 'POS-005', 'Show Ac', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-11-24', 0, 0, '', NULL, '', '', '2019-11-24', '10:23:05', 144, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(34, 4, 'POS-006', 'Hide Ac', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2019-11-24', 0, 0, '', NULL, '', '', '2019-11-24', '10:23:26', 144, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(35, 1, '000-4', 'MICHAL RAYAAPPN', '', 'Male', '', '', 'aluthkade', 'undefined', NULL, '', 0, '', '2020-01-23', 40000, 6, '', NULL, '123', '123', '2025-11-11', '01:24:16', 153, 'NO', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(36, 1, '020', 'M.M.Sabry Ahmed', '', 'Male', '0778262962', '', 'Gampola', 'undefined', NULL, '', 0, '', '2020-07-22', 12500, 0, '', NULL, '123', 'Sabry87', '2020-11-21', '11:33:08', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(37, 1, '021', 'M H Saheel', '', 'Male', '077 574 009', '', 'Kalutara', 'undefined', NULL, '', 0, '', '2020-03-02', 0, 0, '', NULL, '', '', '2020-07-27', '09:39:40', 49, 'YES', 'Level 1', '0000-00-00', 'YES', 'YES', 1, '', 'NEW', NULL, 0),
(38, 1, '022', 'Shazni', '', 'Male', '077123456', '', 'Colombo 6', 'undefined', NULL, '', 0, '', '2020-07-29', 0, 0, '3', NULL, '', '', '2021-02-25', '03:55:31', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(39, 1, '023', 'Abv', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2020-09-13', 0, 0, '', NULL, '', '', '2024-05-28', '02:31:43', 49, 'YES', 'Manager', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(40, 1, '2444', 'Shabith', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2021-02-10', 0, 0, '', NULL, '', '', '2021-02-10', '11:37:27', 158, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(41, 1, '2445', 'Ashik', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2021-02-10', 0, 0, '', NULL, '', '', '2021-02-10', '11:38:18', 158, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(42, 1, '024', 'Naleem', '', 'Male', '077125646', '', '', 'undefined', '2000-10-04', '56-2025,69-2025,102-2025,25-2025,42-2025,29-2025,90-2025,3-2025', 0, '', '2021-10-29', 0, 0, '10', NULL, '', '', '2025-10-13', '10:06:20', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', 'null', 0),
(43, 1, 'naleem', 'naleem', '', 'Male', '077125646', '', '', 'undefined', NULL, '', 0, '', '2021-12-06', 0, 0, '', NULL, '', '', '2021-12-06', '01:36:20', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(44, 1, 'fiverr007', 'bhal007', '', 'Male', '', '', '', 'undefined', NULL, '', 0, '', '2021-12-17', 0, 0, '', NULL, '', '', '2021-12-17', '09:11:46', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(46, 7, '1000', 'Powersoft Pvt Ltd', '', '', '', 'halirramzi@gmail.com', '', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '2022-08-24', '00:20:22', 0, 'YES', 'None', '0000-00-00', 'NO', 'YES', 1, '', 'Up', NULL, 0),
(47, 1, 'cr1', 'Mohamed', '', 'Male', '', '', 'Colombo', 'undefined', NULL, '', 0, '', '2022-09-23', 0, 0, '', NULL, '123', 'Mohamed', '2022-09-23', '07:32:40', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(48, 1, '789', 'Present Solutions', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2023-06-10', 0, 0, '', NULL, '', '', '2023-06-10', '09:40:12', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(49, 1, 'N001', 'HSP Munchee', '', 'Male', '', '', '', '1', NULL, '', 0, '', '2023-08-12', 0, 0, '', NULL, '', '', '2023-08-12', '01:00:47', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(50, 1, 'N002', 'Damith', '', 'Male', '', '', '', '2000', NULL, '', 0, '', '2023-08-23', 0, 0, '', NULL, '', '', '2023-08-23', '03:31:45', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(51, 2, '00002', 'BR22', '', 'Male', '', '', '', '..', NULL, '', 0, '', '0000-00-00', 0, 0, '10', NULL, '', '', '2023-03-04', '09:44:57', 0, 'YES', 'Servicemen', '0000-00-00', 'NO', 'NO', 1, '', 'NEW', NULL, 0),
(52, 1, '998899', 'Kasun Taraka', '', 'Male', '', '', 'asdfsf', '93003371V', NULL, '', 0, '', '2023-08-28', 0, 0, '', NULL, '', '', '2023-08-28', '02:53:00', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(53, 1, '11223344', 'Kaalidaasa', '', 'Male', '0770880044', '', 'Colombo', '930063371V', NULL, '', 0, 'kalidasa@msn.com', '2023-09-12', 0, 0, '', NULL, '', '', '2023-10-30', '10:19:52', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(54, 1, '31947', 'MANIKKAM ROHAN - MR-TRD', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2023-09-27', 0, 0, '', NULL, '', '', '2023-09-27', '01:03:14', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(55, 1, 'test1', 'Test', '', 'Male', '', '', '', '1', NULL, '', 0, '', '2023-10-16', 0, 0, '5', NULL, '', '', '2024-06-06', '02:47:56', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(56, 1, '9300633', 'Halir Ramzi', '', 'Male', '', '', '5656', '930063371V', '2025-10-04', '9300633,565666565,173,173-2025,56-2025,102,102-2025,69-2025,25-2025,42-2025,29-2025,90-2025,3-2025', 0, '', '2023-10-30', 0, 0, '', NULL, '', '', '2023-10-30', '11:15:11', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(57, 1, '666333666', 'jksdjfkjs skdjfksjf', '', 'Male', '', '', 'ssdfsf', '930063371V', NULL, '', 0, '', '2023-10-30', 0, 0, '', NULL, '', '', '2023-10-30', '11:17:09', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(58, 1, '5556666555', 'ssdfsdf', '', 'Male', '', '', 'fsdfsdf', '930063371V', NULL, '', 0, '', '2023-10-30', 0, 0, '', NULL, '', '', '2023-10-30', '11:22:23', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(59, 1, '66665555885', 'sdfsdf', '', 'Male', '', '', 'fsdfsdf', '930063371V', NULL, '', 0, '', '2023-10-30', 0, 0, '', NULL, '', '', '2023-10-30', '11:22:56', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(60, 1, 'Abans', 'Abans', '', 'Male', '', '', '', '12', NULL, '', 0, '', '2023-12-05', 0, 0, '', NULL, '', '', '2024-01-11', '02:41:45', 49, 'YES', 'Manager', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(61, 1, '909209', 'Abdhul Azeez', '', 'Male', '', '', '', '20032093204109', NULL, '', 0, '', '2023-12-20', 0, 0, '', NULL, '', '', '2023-12-20', '09:49:52', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(62, 1, 'S001', 'champika', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2023-12-20', 0, 0, '', NULL, '', '', '2023-12-20', '03:34:33', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(63, 1, 'R001', 'Akash - Servicemen', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2024-03-01', 0, 0, '', NULL, '', '', '2025-10-07', '09:51:35', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(64, 8, '0001', 'Admin', '', '', '', '', '', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'YES', 1, '', 'Up', NULL, 0),
(65, 1, 'R002', 'Repair Package', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2024-06-08', 0, 0, '', NULL, '', '', '2024-06-08', '10:01:00', 163, 'YES', 'Manager', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(66, 3, '1000', 'Naleem', '', 'Male', '077125646', '', '', '1', NULL, '', 0, '', '2024-06-08', 0, 0, '', NULL, '', '', '2024-06-08', '12:32:15', 109, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(67, 4, '0002', 'Naleem2', '', 'Male', '', '', '', '1', NULL, '', 0, '', '2024-06-08', 0, 0, '', NULL, '', '', '2024-06-08', '12:37:05', 151, 'YES', 'Manager', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(68, 1, '123', 'ayra', '', 'Female', '000', '', '000', '000', NULL, '', 0, '', '2024-06-27', 0, 0, '', NULL, '', '', '2024-06-27', '04:00:57', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, '', 'NEW', NULL, 0),
(69, 1, '565666565', '6565', '', 'Male', '', '', '6565', '6565', NULL, '', 0, '', '2024-08-02', 0, 0, '', NULL, '', '', '2024-08-02', '03:13:15', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, '', 'NEW', NULL, 0),
(70, 1, '077088', 'Infas', '', 'Male', '', '', 'Colombo', '930063371V', NULL, '', 665, '', '2024-08-02', 0, 0, '2.5', NULL, '', '', '2025-01-07', '08:49:34', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(71, 1, '56565656555', 'sdfsdf', '', 'Male', '', '', 'sdfsdf', '565565V', NULL, '', 0, '', '2024-08-02', 0, 0, '', NULL, '', '', '2024-08-02', '04:27:24', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, '', 'NEW', NULL, 0),
(72, 1, '0770880044', 'Mohamed Rifkan', '', 'Male', '', '', 'Colombo', '950063371V', NULL, '', 0, '', '2024-08-02', 0, 0, '', NULL, '', '', '2024-08-02', '04:55:56', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, '', 'NEW', '003,006', 0),
(73, 1, '', '', '', '', '', '', '', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 0, '', 'Up', NULL, 0),
(74, 1, 'NT001', 'Ntrade', '', 'Male', '', '', '', '123', NULL, '', 0, '', '2024-08-15', 0, 0, '', NULL, '', '', '2024-08-15', '09:18:11', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '002,003', 'NEW', NULL, 0),
(75, 1, '1101105656', 'Kausf ', '', 'Male', '656656', '', 'Colombo', 'kjk5565V', NULL, '', 0, '6565@gmail.com', '2024-08-17', 0, 0, '', NULL, '5656', '', '2024-08-17', '08:25:27', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(76, 1, '0775939524', 'Raza Hussain', '', 'Male', '0754733100', '', 'Colombo', '930063371V', NULL, '', 0, 'raza@gmail.com', '2024-08-28', 0, 0, '', NULL, '', '', '2024-08-28', '08:48:24', 138, 'YES', 'Level 1', '0000-00-00', 'YES', 'YES', 0, 'null', 'NEW', NULL, 0),
(77, 1, '005566', 'Kamal Silwa', '', 'Male', '07777777', '', 'Colombo', '930063311V', NULL, '', 0, 'kamal@gmail.com', '2024-10-21', 0, 0, '', NULL, '', '', '2024-10-21', '05:30:30', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(78, 1, '100', 'Aysha Silma', '', 'Female', '145', '', '111', '222', NULL, '', 0, '', '2024-06-12', 1000, 19, '-1', NULL, '', '', '2024-11-25', '06:10:43', 0, 'YES', 'null', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(79, 1, '0009', 'simaya', '', 'Male', '456', '', '.', '1', NULL, '', 0, '', '2024-06-12', 24000, 17, '5', NULL, '', '', '2024-12-06', '11:41:48', 0, 'YES', 'null', '0000-00-00', 'NO', 'YES', 1, '', 'NEW', NULL, 0),
(80, 1, '565656565', 'Aathif', '', 'Male', '0656565', '', 'Qatar', '930063371V', NULL, '', 0, 'aathif@gmail.com', '2024-11-01', 0, 0, '', NULL, '', '', '2025-10-23', '03:23:34', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(81, 1, 'D-001', 'Demo Account', '', 'Male', '', '', '', '004', NULL, '', 0, '', '2024-11-07', 0, 0, '', NULL, '', '', '2025-11-06', '03:31:08', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(82, 1, 'C-1124', 'Cement', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2024-11-28', 0, 0, '', NULL, '', '', '2024-11-28', '08:47:18', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(83, 7, '1001', 'Naleem', '', 'Male', '077125646', '', '', '.', NULL, '', 0, '', '2024-12-06', 0, 0, '', NULL, '', '', '2024-12-06', '10:40:37', 167, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(84, 7, '1002', 'Infaz', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2024-12-06', 0, 0, '', NULL, '', '', '2024-12-06', '10:40:55', 167, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(85, 1, 'C-1125', 'Rifadha ', '', 'Male', '0757303922', '', 'colombo 03', '123456', NULL, '', 0, '', '2024-12-06', 0, 0, '', NULL, '', '', '2024-12-06', '07:31:03', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(86, 1, 'SLNDP010', 'APSARA HARDWARE', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2024-12-11', 0, 0, '', NULL, '', '', '2024-12-11', '11:05:31', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(87, 1, 'SLNDP011', 'HUSNA', '', 'Female', '0750710837', '', '', '200268002755', NULL, '', 0, '', '2024-12-18', 0, 0, '', NULL, '', '', '2024-12-18', '06:18:20', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(88, 1, '566', 'mobile', '', '', '', '', ' ', '', NULL, '', 0, '', '0000-00-00', 0, 0, NULL, NULL, '', '', '0000-00-00', '00:00:00', 0, 'YES', 'None', '0000-00-00', 'NO', 'NO', 0, '', 'Up', NULL, 0),
(89, 1, '1777', 'ORDER/ DELIVERY ', '', 'Male', '', '', '', '00', NULL, '', 0, '', '2025-01-04', 0, 0, '', NULL, '', '', '2025-01-04', '05:34:56', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(90, 1, '17778', 'Aysha', '', 'Male', '', '', '', '222', NULL, '', 0, '', '2025-01-07', 0, 0, '', NULL, '', '', '2025-01-07', '07:16:46', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(91, 1, '1414', 'Azeem', '', 'Male', '', '', 'Colombo', '455882955', NULL, '', 0, '', '2025-01-10', 0, 0, '', NULL, '', '', '2025-01-10', '10:19:06', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(92, 1, '1415', 'Muwah ', '', 'Female', '', '', '', '200480202613', NULL, '', 0, '', '2025-02-01', 0, 0, '', NULL, '', '', '2025-02-01', '05:34:48', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(93, 1, '1416', 'mariyam sinan ', '', 'Male', '', '', '', '20055557463', NULL, '', 0, '', '2025-02-20', 0, 0, '', NULL, '', '', '2025-02-20', '01:16:32', 195, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(94, 1, '141414', 'Azeem Ali', '', 'Male', '0764477446', '', '544, 2nd Flr., Maradana Rd., Colombo 10.', '722580966V', NULL, '', 0, 'azeem@go2xpower.com', '1999-01-01', 0, 1, '35', NULL, '004', 'AzeemAli', '2025-02-26', '07:06:43', 49, 'YES', 'Sales Manager', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(95, 1, 'AS001', 'Mr. Alsan', '', 'Male', '', '', '', '.', '1988-10-01', '9300633,A007,56-2025', 0, '', '2025-02-27', 0, 0, '', NULL, '', '', '2025-02-27', '07:09:07', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(96, 1, 'AS002', 'fathima hafsa', '', 'Female', '', '', '', '200453010984', NULL, '', 0, '', '2025-03-10', 0, 0, '', NULL, '', '', '2025-03-10', '11:21:21', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(97, 1, 'AS003', 'kadeeja', '', 'Female', '', '', '', '12345678911', NULL, '', 0, '', '2025-03-10', 0, 0, '', NULL, '', '', '2025-03-10', '11:22:25', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(98, 1, 'S004', 'Muwah ', '', 'Female', '', '', '', '200480202613', NULL, '', 0, '', '2025-03-17', 0, 0, '', NULL, '', '', '2025-03-17', '07:43:34', 161, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(99, 1, 'AS004', 'Haniyya', '', 'Female', '0552367485', '', 'ark', '1058626399454', NULL, '', 0, '', '2025-03-17', 0, 0, '', NULL, '', '', '2025-03-17', '07:44:30', 136, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(100, 1, 'A005', 'BASITH', '', 'Male', '', '', '', '1', NULL, '', 0, '', '2025-03-19', 0, 0, '', NULL, '', '', '2025-03-19', '05:26:09', 136, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(101, 1, 'A006', 'New Staff', 'New', 'Male', '', '', '', '123', NULL, '', 0, '', '2025-05-19', 0, 0, '', NULL, '', '', '2025-05-19', '06:26:11', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(102, 1, 'A007', 'AMASHI', '', 'Female', '', '', '', '200369710136', NULL, '', 0, '', '2025-09-03', 0, 0, '', NULL, '', '', '2025-09-03', '09:09:10', 197, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(103, 1, 'A008', 'INDRANI', '', 'Female', '', '', '', '123', NULL, '', 0, '', '2025-09-03', 0, 0, '', NULL, '', '', '2025-09-03', '09:17:16', 207, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(104, 1, '120322', 'Amashi', '', 'Male', '', '', '', '4155666', NULL, '', 0, '', '2025-09-04', 0, 0, '', NULL, '', '', '2025-09-04', '11:12:47', 207, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(105, 1, '9300633', 'Abrar Munawfer', 'MMM Abrar', 'Male', '0776817476', '', '130A samad MW Massala Beruwala', '200016701966', '2000-10-01', '9300633,A007,56-2025', 0, 'abrarmunawfer.ebay@gmail.com', '2025-09-22', 0, 0, '', NULL, '200016', 'abrarm', '2025-09-22', '01:34:31', 173, 'YES', 'Sales Manager', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(106, 1, '9300634', 'Abrar User', 'MMM Abrar', 'Male', '0776817476', '', '130A samad MW Massala Beruwala', '200016701966', NULL, '', 0, 'abrarmunawfer.ebay@gmail.com', '2025-09-30', 0, 0, '', NULL, '', '', '2025-09-30', '11:12:26', 173, 'YES', 'Manager', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(107, 1, '4567', 'Fathima', 'Fathi Fathima', 'Male', '077752896', '', '12 Galle road, Colombo ', '20004582582', NULL, '', 0, '', '2025-10-08', 0, 0, '', NULL, '', '', '2025-10-08', '02:09:26', 162, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(108, 1, '10000000', 'Dilani ', '', 'Female', '', '', '', '123456898', NULL, '', 0, '', '2025-10-13', 0, 0, '', NULL, '', '', '2025-10-13', '09:20:13', 195, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(109, 1, '32500', 'dasuni', '', 'Female', '021545521', '', '', '251625662', NULL, '', 0, '', '2025-10-14', 0, 0, '', NULL, '', '', '2025-10-14', '08:28:29', 213, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(110, 1, '5252', 'Azmy', 'Azmy', 'Male', '', '', '', '87897', NULL, '', 0, '', '2025-10-30', 0, 0, '', NULL, '', '', '2025-10-30', '10:00:03', 199, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(111, 1, '1234', 'AAYSHA', '', 'Male', '', '', '', '00000001V', NULL, '', 0, '', '2025-11-07', 0, 0, '', NULL, '', '', '2025-11-14', '01:36:38', 195, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, '111', 'NEW', NULL, 0),
(112, 1, '001', 'IMRAN', '', 'Male', '0112337179', '', '100/24 mumtaz mahal 1st cross street pettah ', '200', NULL, '', 0, '', '2025-11-13', 0, 0, '', NULL, '', '', '2025-11-20', '01:40:11', 206, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(113, 1, 'NL001', 'Naleem Hasheem', '', 'Male', '', '', '', '.', NULL, '', 0, '', '2025-12-01', 0, 0, '', NULL, '', '', '2025-12-01', '10:26:23', 163, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(114, 1, '0014', 'SHABRA NIJAM', '', 'Female', '', '', '', '200266600250', NULL, '', 0, '', '2025-12-01', 0, 0, '', NULL, '', '', '2025-12-01', '10:44:31', 49, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(115, 1, 'u001', 'mohamed', '', 'Male', '0722693627', '', 'beruwala ', '.', NULL, '', 0, '', '2025-12-01', 0, 1, '', NULL, '2006@umair', 'umair@2006', '2025-12-01', '10:47:11', 49, 'YES', 'Level 3', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(116, 1, '062', 'M.S.F Aaysha', '', 'Female', '', '', '', '200467502533', NULL, '', 0, '', '2025-12-01', 0, 0, '', NULL, '', '', '2025-12-01', '10:49:19', 216, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(117, 1, '072', 'umair ', 'mohamed umair', 'Male', '0722693617', '', 'beruwala', '.', NULL, '', 0, '', '2025-12-01', 0, 1, '6', NULL, '004', 'mohamed', '2025-12-01', '11:02:04', 49, 'YES', 'Level 3', '0000-00-00', 'YES', 'YES', 1, 'null', 'NEW', NULL, 0),
(118, 1, 'SN12', 'THOMAS', '', 'Male', '', '', '', '2', NULL, '', 0, '', '2025-12-01', 0, 0, '', NULL, '', '', '2025-12-01', '11:15:31', 217, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 1, 'null', 'NEW', NULL, 0),
(119, 1, '01', 'aadhil', '', 'Male', '', '', '', '200769362624', NULL, '', 0, '', '2025-12-05', 0, 0, '', NULL, '', '', '2025-12-05', '03:54:19', 187, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(120, 1, '011011', 'Thalha Khaan', 'Thalha', 'Male', '', '', 'Colombo', '930063371V', NULL, '', 0, '', '2025-12-09', 0, 0, '', NULL, '', '', '2025-12-09', '11:32:24', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0),
(121, 1, '022022', 'AAA Rasik', 'Raaz', 'Male', '', '', 'Colombo', '930063317V', NULL, '', 0, '', '2025-12-10', 0, 0, '', NULL, '', '', '2025-12-10', '09:25:37', 138, 'YES', 'Level 1', '0000-00-00', 'NO', 'YES', 0, 'null', 'NEW', NULL, 0);

-- --------------------------------------------------------

--
-- Table structure for table `user_activity`
--

CREATE TABLE `user_activity` (
  `ID` int(11) NOT NULL,
  `salesrepTb` int(11) NOT NULL,
  `activity_type` varchar(100) NOT NULL,
  `duration` int(11) NOT NULL,
  `rDateTime` datetime NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1 COLLATE=latin1_swedish_ci;

-- --------------------------------------------------------

--
-- Table structure for table `web_images`
--

CREATE TABLE `web_images` (
  `ID` int(11) NOT NULL,
  `br_id` int(11) NOT NULL,
  `imgID` int(11) NOT NULL,
  `imgName` varchar(2000) NOT NULL,
  `itmName` varchar(200) NOT NULL,
  `type` varchar(50) NOT NULL,
  `user_id` int(11) NOT NULL,
  `date` date NOT NULL,
  `time` time NOT NULL,
  `status` varchar(200) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=latin1 COLLATE=latin1_swedish_ci;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `salesrep`
--
ALTER TABLE `salesrep`
  ADD PRIMARY KEY (`ID`),
  ADD KEY `ID` (`ID`),
  ADD KEY `br_id` (`br_id`,`RepID`);

--
-- Indexes for table `user_activity`
--
ALTER TABLE `user_activity`
  ADD PRIMARY KEY (`ID`);

--
-- Indexes for table `web_images`
--
ALTER TABLE `web_images`
  ADD PRIMARY KEY (`ID`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `salesrep`
--
ALTER TABLE `salesrep`
  MODIFY `ID` int(11) NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=122;

--
-- AUTO_INCREMENT for table `user_activity`
--
ALTER TABLE `user_activity`
  MODIFY `ID` int(11) NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `web_images`
--
ALTER TABLE `web_images`
  MODIFY `ID` int(11) NOT NULL AUTO_INCREMENT;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
