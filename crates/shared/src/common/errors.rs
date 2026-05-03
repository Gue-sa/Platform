use crate::{bitpacker::BitPacker, common::types::AisPacket, satcom_message::SatComMessage};
use std::{io::Error, time::SystemTimeError};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

// --- ERREURS DE BAS NIVEAU ---

#[derive(Error, Debug, Clone)]
pub enum ClockError {
    #[error("Erreur de temps système")]
    SystemTime(#[from] SystemTimeError),
    #[error("Dépassement du slot (overshoot)")]
    SlotOvershoot,
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum BitPackerError {
    #[error("Index hors limites")]
    IndexOutOfBounds,
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum BoatInfoError {
    #[error("Données statiques corrompues (RwLock empoisonné)")]
    StaticDataPoisoned,
    #[error("Données de voyage corrompues (RwLock empoisonné)")]
    VoyageDataPoisoned,
    #[error("Données de navigation corrompues (RwLock empoisonné)")]
    NavigationDataPoisoned,
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum SlotsMapError {
    #[error("Carte des slots corrompue (RwLock empoisonné)")]
    SlotsMapPoisoned,
}

// --- ERREURS AIS & MESSAGERIE ---

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum CommunicationStateError {
    #[error("Timeout SOTDMA manquant")]
    NoSotdmaTimeout,
    #[error("Offset de slot SOTDMA manquant")]
    NoSotdmaSlotOffset,
    #[error("Heure UTC manquante")]
    NoUtcHour,
    #[error("Minute UTC manquante")]
    NoUtcMinute,
    #[error("Numéro de slot ITDMA manquant")]
    NoItdmaSlotNumber,
    #[error("Stations reçues SOTDMA manquantes")]
    NoSotdmaReceivedStations,
    #[error("Flag de maintien ITDMA manquant")]
    NoItdmaKeepFlag,
    #[error("Incrément de slot ITDMA manquant")]
    NoItdmaSlotIncrement,
    #[error("Nombre de slots ITDMA manquant")]
    NoItdmaNumberOfSlots,
    #[error("Timeout SOTDMA inconnu")]
    UnkownSotdmaTimeout,
    #[error("Type de message inconnu")]
    UnknownMessageType,
    #[error("Erreur BitPacker")]
    BitPacker(#[from] BitPackerError),
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum AisMessageError {
    #[error("Type de message AIS inconnu")]
    MessageTypeNotImplemented,
    #[error("Timeout SOTDMA inconnu")]
    UnkownSotdmaTimeout,
    #[error("Erreur CRC")]
    CrcMismatch,
    #[error("État de communication manquant")]
    NoCommunicationState,
    #[error(transparent)]
    BitPacker(#[from] BitPackerError),
    #[error(transparent)]
    CommunicationState(#[from] CommunicationStateError),
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
}

#[derive(Error, Debug, Clone)]
pub enum AisError {
    #[error("Message auto-émis ignoré")]
    SelfEmittedMessage,
    #[error("Aucun slot libre disponible")]
    NoFreeSlot,
    #[error("Aucun slot possédé trouvé")]
    NoOwnedSlot,
    #[error("Sélection de slot invalide")]
    NoValidSlotSelection,
    #[error("Échec de l'initialisation SOTDMA")]
    SotdmaInitFailed,
    #[error(transparent)]
    CommunicationState(#[from] CommunicationStateError),
    #[error(transparent)]
    AisMessage(#[from] AisMessageError),
    #[error(transparent)]
    Clock(#[from] ClockError),
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
    #[error(transparent)]
    SlotsMap(#[from] SlotsMapError),
    #[error(transparent)]
    BoatsRegistry(#[from] BoatsRegistryError),
    #[error("Erreur d'envoi MPSC (BitPacker)")]
    SendError(#[from] SendError<BitPacker>),
}

// --- ERREURS SATCOM & VOYAGE ---

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum VoyageOrderError {
    #[error("Ordre de voyage malformé")]
    MalformedVoyageOrder,
    #[error(transparent)]
    BitPacker(#[from] BitPackerError),
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum SatComMessageError {
    #[error("Type de message SatCom inconnu")]
    UnknownSatComMessageType,
    #[error(transparent)]
    BitPacker(#[from] BitPackerError),
    #[error(transparent)]
    VoyageOrder(#[from] VoyageOrderError),
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum SatComError {
    #[error("Type de message inconnu")]
    UnknownMessageType,
    #[error(transparent)]
    SatComMessage(#[from] SatComMessageError),
}

// --- ERREURS SYSTÈME & INFRA ---

#[derive(Error, Debug, PartialEq)]
pub enum DatabaseManagerError {
    #[error("Date Naive invalide")]
    InvalidNaiveDate,
    #[error("Erreur d'insertion : {0}")]
    InsertionError(diesel::result::Error),
    #[error("Erreur de requête : {0}")]
    QueryError(diesel::result::Error),
    #[error("Erreur de mise à jour : {0}")]
    UpdateError(diesel::result::Error),
    #[error("Erreur de suppression : {0}")]
    DeletionError(diesel::result::Error),
}

#[derive(Error, Debug)]
pub enum AntennaError {
    #[error("Échec initialisation antenne")]
    InitError(#[from] Error),
    #[error("Erreur d'émission radio")]
    EmissionError,
    #[error("Erreur de réception radio")]
    ReceptionError,
    #[error("Erreur d'envoi MPSC (AisPacket)")]
    SendAisPacketError(#[from] SendError<AisPacket>),
    #[error("Erreur d'envoi MPSC (BitPacker)")]
    SendBitPackerError(#[from] SendError<BitPacker>),
}

#[derive(Error, Debug)]
pub enum RadioBuilderError {
    #[error(transparent)]
    Antenna(#[from] AntennaError),
}

#[derive(Error, Debug)]
pub enum HarbourmasterError {
    #[error(transparent)]
    DatabaseManager(#[from] DatabaseManagerError),
    #[error(transparent)]
    RadioBuilder(#[from] RadioBuilderError),
    #[error(transparent)]
    Antenna(#[from] AntennaError),
}

#[derive(Error, Debug)]
pub enum BoatError {
    #[error(transparent)]
    RadioBuilder(#[from] RadioBuilderError),
    #[error(transparent)]
    Antenna(#[from] AntennaError),
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
}

#[derive(Error, Debug)]
pub enum BoardComputerError {
    #[error("Aucun ordre de voyage actif")]
    NoVoyageOrder,
    #[error("Aucune révision d'ordre de voyage")]
    NoVoyageOrderRevision,
    #[error("Erreur d'envoi MPSC (SatComMessage)")]
    SendError(#[from] SendError<SatComMessage>),
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
}

#[derive(Error, Debug)]
pub enum FmsError {
    #[error("Gestionnaire de base de données empoisonné")]
    DatabaseManagerPoisoned,
    #[error(transparent)]
    DatabaseManager(#[from] DatabaseManagerError),
    #[error("Erreur d'envoi MPSC (SatComMessage)")]
    SendError(#[from] SendError<SatComMessage>),
    #[error(transparent)]
    BoatsRegistry(#[from] BoatsRegistryError),
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
}

// --- REGISTRES & FRÉQUENCES ---

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum ClientsRegistryError {
    #[error("Registre des clients corrompu (RwLock empoisonné)")]
    ClientsRegistryPoisoned,
}

#[derive(Error, Debug, Clone, PartialEq, Copy)]
pub enum BoatsRegistryError {
    #[error("MMSI inconnu")]
    UnkownMmsi,
    #[error("MMSI déjà enregistré")]
    MmsiAlreadyRegistered,
    #[error(transparent)]
    BoatInfo(#[from] BoatInfoError),
}

#[derive(Error, Debug)]
pub enum RadioFrequencyError {
    #[error("Erreur d'envoi")]
    SendError,
    #[error(transparent)]
    ClientsRegistry(#[from] ClientsRegistryError),
    #[error(transparent)]
    BitPacker(#[from] BitPackerError),
}

// Implémentation manuelle car le type générique de SendError varie
impl From<SendError<&[u8]>> for RadioFrequencyError {
    fn from(_: SendError<&[u8]>) -> Self {
        Self::SendError
    }
}

// --- ALIAS ---

pub type ClockResult<T> = Result<T, ClockError>;
pub type BitPackerResult<T> = Result<T, BitPackerError>;
pub type AisResult<T> = Result<T, AisError>;
pub type AisMessageResult<T> = Result<T, AisMessageError>;
pub type CommunicationStateResult<T> = Result<T, CommunicationStateError>;
pub type VoyageOrderResult<T> = Result<T, VoyageOrderError>;
pub type SatComResult<T> = Result<T, SatComError>;
pub type SatComMessageResult<T> = Result<T, SatComMessageError>;
pub type DatabaseManagerResult<T> = Result<T, DatabaseManagerError>;
pub type HarbourmasterResult<T> = Result<T, HarbourmasterError>;
pub type AntennaResult<T> = Result<T, AntennaError>;
pub type RadioBuilderResult<T> = Result<T, RadioBuilderError>;
pub type BoatResult<T> = Result<T, BoatError>;
pub type BoardComputerResult<T> = Result<T, BoardComputerError>;
pub type FmsResult<T> = Result<T, FmsError>;
pub type ClientsRegistryResult<T> = Result<T, ClientsRegistryError>;
pub type BoatsRegistryResult<T> = Result<T, BoatsRegistryError>;
pub type RadioFrequencyResult<T> = Result<T, RadioFrequencyError>;
pub type BoatInfoResult<T> = Result<T, BoatInfoError>;
pub type SlotsMapResult<T> = Result<T, SlotsMapError>;
