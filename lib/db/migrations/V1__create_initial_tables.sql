CREATE TABLE IF NOT EXISTS `block` (
  `idBlockPk`      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `number`         BIGINT UNSIGNED NOT NULL,
  `hash`           VARCHAR(66) NOT NULL,
  `parentHash`     VARCHAR(66) NOT NULL,
  `timestamp`      BIGINT UNSIGNED NOT NULL,
  `miner`          VARCHAR(42) NOT NULL,
  `gasUsed`        BIGINT UNSIGNED NOT NULL,
  `gasLimit`       BIGINT UNSIGNED NOT NULL,
  `baseFeePerGas`  VARCHAR(80),
  `size`           BIGINT UNSIGNED NOT NULL,
  `txCount`        INT UNSIGNED NOT NULL,
  `extraData`      TEXT,
  CONSTRAINT `uqBlockNumber` UNIQUE (`number`),
  CONSTRAINT `uqBlockHash` UNIQUE (`hash`)
);

CREATE INDEX `idxBlockTimestamp` ON `block` (`timestamp`);

CREATE TABLE IF NOT EXISTS `transaction` (
  `idTransactionPk`       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `hash`                  VARCHAR(66) NOT NULL,
  `blockNumber`           BIGINT UNSIGNED NOT NULL,
  `txIndex`               INT UNSIGNED NOT NULL,
  `fromAddress`           VARCHAR(42) NOT NULL,
  `toAddress`             VARCHAR(42),
  `value`                 VARCHAR(80) NOT NULL,
  `gas`                   BIGINT UNSIGNED NOT NULL,
  `gasPrice`              VARCHAR(80),
  `maxFeePerGas`          VARCHAR(80),
  `maxPriorityFeePerGas`  VARCHAR(80),
  `nonce`                 BIGINT UNSIGNED NOT NULL,
  `txType`                SMALLINT UNSIGNED NOT NULL,
  `input`                 MEDIUMTEXT NOT NULL,
  `status`                TINYINT UNSIGNED,
  `gasUsed`               BIGINT UNSIGNED,
  `cumulativeGasUsed`     BIGINT UNSIGNED,
  `effectiveGasPrice`     VARCHAR(80),
  `contractAddress`       VARCHAR(42),
  CONSTRAINT `uqTransactionHash` UNIQUE (`hash`),
  CONSTRAINT `fkTransactionBlock` FOREIGN KEY (`blockNumber`) REFERENCES `block` (`number`)
);

CREATE INDEX `idxTransactionBlockNumber` ON `transaction` (`blockNumber`);
CREATE INDEX `idxTransactionFrom` ON `transaction` (`fromAddress`);
CREATE INDEX `idxTransactionTo` ON `transaction` (`toAddress`);

CREATE TABLE IF NOT EXISTS `log` (
  `idLogPk`      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `blockNumber`  BIGINT UNSIGNED NOT NULL,
  `logIndex`     INT UNSIGNED NOT NULL,
  `txHash`       VARCHAR(66) NOT NULL,
  `txIndex`      INT UNSIGNED NOT NULL,
  `address`      VARCHAR(42) NOT NULL,
  `topic0`       VARCHAR(66),
  `topic1`       VARCHAR(66),
  `topic2`       VARCHAR(66),
  `topic3`       VARCHAR(66),
  `data`         MEDIUMTEXT NOT NULL,
  `removed`      BOOLEAN NOT NULL DEFAULT 0,
  CONSTRAINT `uqLogBlockIndex` UNIQUE (`blockNumber`, `logIndex`),
  CONSTRAINT `fkLogBlock` FOREIGN KEY (`blockNumber`) REFERENCES `block` (`number`)
);

CREATE INDEX `idxLogTxHash` ON `log` (`txHash`);
CREATE INDEX `idxLogAddress` ON `log` (`address`);
CREATE INDEX `idxLogTopic0` ON `log` (`topic0`);
CREATE INDEX `idxLogAddressTopic0` ON `log` (`address`, `topic0`);

CREATE TABLE IF NOT EXISTS `contract` (
  `idContractPk`       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `address`            VARCHAR(42) NOT NULL,
  `creatorAddress`     VARCHAR(42) NOT NULL,
  `creationTxHash`     VARCHAR(66) NOT NULL,
  `creationBlock`      BIGINT UNSIGNED NOT NULL,
  `creationTimestamp`  BIGINT UNSIGNED NOT NULL,
  `creationStatus`     TINYINT UNSIGNED,
  CONSTRAINT `uqContractAddress` UNIQUE (`address`)
);

CREATE INDEX `idxContractCreator` ON `contract` (`creatorAddress`);
CREATE INDEX `idxContractBlock` ON `contract` (`creationBlock`);

CREATE TABLE IF NOT EXISTS `address` (
  `idAddressPk`     BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `address`         VARCHAR(42) NOT NULL,
  `firstSeenBlock`  BIGINT UNSIGNED NOT NULL,
  `lastSeenBlock`   BIGINT UNSIGNED NOT NULL,
  `isContract`      BOOLEAN NOT NULL DEFAULT 0,
  CONSTRAINT `uqAddressAddress` UNIQUE (`address`)
);

CREATE INDEX `idxAddressFirstSeen` ON `address` (`firstSeenBlock`);
CREATE INDEX `idxAddressIsContract` ON `address` (`isContract`);

CREATE TABLE IF NOT EXISTS `token` (
  `idTokenPk`       BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `address`         VARCHAR(42) NOT NULL,
  `tokenType`       VARCHAR(10) NOT NULL,
  `name`            VARCHAR(255),
  `symbol`          VARCHAR(255),
  `decimals`        SMALLINT UNSIGNED,
  `firstSeenBlock`  BIGINT UNSIGNED NOT NULL,
  CONSTRAINT `uqTokenAddress` UNIQUE (`address`)
);

CREATE INDEX `idxTokenType` ON `token` (`tokenType`);
CREATE INDEX `idxTokenSymbol` ON `token` (`symbol`);

CREATE TABLE IF NOT EXISTS `tokenTransfer` (
  `idTokenTransferPk`  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
  `blockNumber`        BIGINT UNSIGNED NOT NULL,
  `logIndex`           INT UNSIGNED NOT NULL,
  `subIndex`           INT UNSIGNED NOT NULL,
  `txHash`             VARCHAR(66) NOT NULL,
  `txIndex`            INT UNSIGNED NOT NULL,
  `tokenAddress`       VARCHAR(42) NOT NULL,
  `tokenType`          VARCHAR(10) NOT NULL,
  `operator`           VARCHAR(42),
  `fromAddress`        VARCHAR(42) NOT NULL,
  `toAddress`          VARCHAR(42) NOT NULL,
  `tokenId`            VARCHAR(80),
  `amount`             VARCHAR(80) NOT NULL,
  CONSTRAINT `uqTokenTransferBlockIndex` UNIQUE (`blockNumber`, `logIndex`, `subIndex`),
  CONSTRAINT `fkTokenTransferBlock` FOREIGN KEY (`blockNumber`) REFERENCES `block` (`number`)
);

CREATE INDEX `idxTokenTransferTxHash` ON `tokenTransfer` (`txHash`);
CREATE INDEX `idxTokenTransferToken` ON `tokenTransfer` (`tokenAddress`);
CREATE INDEX `idxTokenTransferFrom` ON `tokenTransfer` (`fromAddress`);
CREATE INDEX `idxTokenTransferTo` ON `tokenTransfer` (`toAddress`);
CREATE INDEX `idxTokenTransferTokenId` ON `tokenTransfer` (`tokenAddress`, `tokenId`);
