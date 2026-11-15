// Apollo 11 SysML v2 Model, imported into ArcLang.
//
// Original: Philipp Helle and Gerrit Schramm (Airbus), "Fly me to the Moon -
// Modeling Apollo 11 using SysML v2", https://github.com/airbus/apollo-11-sysml-v2
// at commit 0e8a7d6790cfa7e15b991eb745e9a38f9bed0474, under the Mozilla Public
// License 2.0 (LICENSE-MPL-2.0.md beside this file).
//
// This file is a modification of that model: `arclang import -f sysml` wrote it
// from the SysML v2 text, unchanged except for the model's name. It stays under
// the Mozilla Public License 2.0; the source form of the original is the
// repository above.

model "Apollo 11" {
    name: "Apollo 11"
    description: "Imported from the Apollo 11 SysML v2 model of Airbus"
}

exchange_item "VehicleStatus" {
    id: "VehicleStatus"
    description: "Represents the complete state of a vehicle system, including configuration, resources, and trajectory."
}
exchange_item "CrewStatus" {
    id: "CrewStatus"
    description: "Represents the state of the crew, including health and readiness."
}
exchange_item "MissionPlan" {
    id: "MissionPlan"
    description: "Contains the overall objectives and parameters for the mission."
}
exchange_item "LunarSamples" {
    id: "LunarSamples"
    description: "Represents the collected geological materials from the Moon."
}
exchange_item "ScientificData" {
    id: "ScientificData"
    description: "Represents data gathered from deployed scientific instruments."
}
exchange_item "MissionReport" {
    id: "MissionReport"
    description: "A comprehensive report detailing the outcomes of the mission."
}
exchange_item "PreparationPhaseCompletedNotification" {
    id: "PreparationPhaseCompletedNotification"
}
exchange_item "LaunchPhaseCompletedNotification" {
    id: "LaunchPhaseCompletedNotification"
}
exchange_item "TransLunarInjectionPhaseCompletedNotification" {
    id: "TransLunarInjectionPhaseCompletedNotification"
}
exchange_item "TransLunarCoastPhaseCompletedNotification" {
    id: "TransLunarCoastPhaseCompletedNotification"
}
exchange_item "LunarOrbitInsertionPhaseCompletedNotification" {
    id: "LunarOrbitInsertionPhaseCompletedNotification"
}
exchange_item "LunarOrbitOperationsDescentPreparationPhaseCompletedNotification" {
    id: "LunarOrbitOperationsDescentPreparationPhaseCompletedNotification"
}
exchange_item "PoweredDescentPhaseCompletedNotification" {
    id: "PoweredDescentPhaseCompletedNotification"
}
exchange_item "LunarSurfaceOperationsPhaseCompletedNotification" {
    id: "LunarSurfaceOperationsPhaseCompletedNotification"
}
exchange_item "LunarAscentRendezvousPhaseCompletedNotification" {
    id: "LunarAscentRendezvousPhaseCompletedNotification"
}
exchange_item "TransEarthInjectionPhaseCompletedNotification" {
    id: "TransEarthInjectionPhaseCompletedNotification"
}
exchange_item "TransEarthCoastPhaseCompletedNotification" {
    id: "TransEarthCoastPhaseCompletedNotification"
}
exchange_item "ReentryLandingPhaseCompletedNotification" {
    id: "ReentryLandingPhaseCompletedNotification"
}
exchange_item "RecoveryQuarantinePhaseCompletedNotification" {
    id: "RecoveryQuarantinePhaseCompletedNotification"
}
exchange_item "StructuralLoad" {
    id: "StructuralLoad"
    description: "Represents the transfer of physical forces (compression, tension, torsion)."
}
exchange_item "CommandSignal" {
    id: "CommandSignal"
    description: "Represents a discrete command or electrical signal."
}
exchange_item "TelemetryData" {
    id: "TelemetryData"
    description: "Represents a flow of vehicle health and status data."
}
exchange_item "ElectricalPower" {
    id: "ElectricalPower"
    description: "Represents a flow of electrical energy."
}
exchange_item "LifeSupportFlows" {
    id: "LifeSupportFlows"
    description: "Represents a flow of oxygen, water, and thermal control fluids."
}

type "John Young" {
    description: "John Young had a long and distinguished career, flying in the Gemini, Apollo, and Space Shuttle programs. He was the Command Module Pilot on Apollo 10 and the Commander of Apollo 16, becoming the ninth person to walk on the Moon."
    dateOfBirth: "new Iso8601DateTime(\"1930-09-24\")"
    dateOfDeath: "new Iso8601DateTime(\"2018-01-05\")"
}
type "PrepareVehicleForReentry" {
    description: "Configures the main vehicle for atmospheric entry."
}
type "ExecuteTranspositionManeuver" {
    description: "Pilots the Command/Service Module after separation from the S-IVB stage."
}
type "GuideAscentTrajectory" {
    description: "Autonomously executes the pre-programmed flight plan, controlling engine gimbaling and staging events."
}
type "James Lovell" {
    description: "James \"Jim\" Lovell is one of only three men to have flown to the Moon twice. He flew on Apollo 8, the first mission to orbit the Moon, and was the Commander of the ill-fated Apollo 13 mission, which he and his crew safely returned to Earth after a major in-flight emergency."
    dateOfBirth: "new Iso8601DateTime(\"1928-03-25\")"
}
type "CrawlerTransporter" {
    description: "The mobile platform used to move the rocket from the VAB to the launch pad."
}
type "ProvideAttitudeControlThrust" {
    description: "Fires Reaction Control System thrusters to orient the spacecraft."
}
type "RocketEngine" {
    description: "An abstract representation of a rocket engine, defining its core performance attributes."
}
type "SeparateLanderFromOrbiter" {
    description: "Physically separates the two spacecraft."
}
type "ProvideStage2Thrust" {
    description: "Provides thrust after first stage separation to propel the vehicle to the edge of space."
}
type "F-1" extends "RocketEngine" {
    description: "A single-chamber liquid-propellant rocket engine, the most powerful ever built, used in the S-IC first stage."
    thrustSeaLevel: 6770 kN
    thrustVacuum: 7770 kN
    specificImpulseVacuum: 304 s
}
type "ManuallyPilotLander" {
    description: "Allows for manual control of the lander to avoid hazards."
}
type "DeployDecelerators" {
    description: "Deploys the parachute system to slow the vehicle for landing."
}
type "ControlLunarOrbitInsertionBurn" {
    description: "Guides the LOI burn to achieve the target orbit."
}
type "ExecuteCourseCorrectionBurn" {
    description: "Fires the main engine or RCS thrusters to execute the planned trajectory adjustment."
}
type "ConductSurfaceExploration" {
    description: "The act of the crew operating on the lunar surface."
}
type "ConductSurfaceRestPeriod" {
    description: "Manages the crew rest period while on the lunar surface."
}
type "LaunchControlCenter" {
    description: "The facility for controlling the pre-launch and launch sequences."
}
type "MultistageRocket" {
    description: "An abstract representation of a rocket composed of multiple stages."
}
type "ProvideLunarOrbitInsertionThrust" {
    description: "Fires the main engine to slow the spacecraft for lunar capture."
}
type "RocketStage" extends "FuelledComponent" {
    description: "An abstract representation of a single stage of a rocket, containing engines and propellant."
}
type "DeploySurfaceExperiments" {
    description: "Sets up scientific instrument packages on the lunar surface."
}
type "MonitorDescent" {
    description: "Tracks altitude and velocity telemetry during landing."
}
type "SeparateReturnCapsule" {
    description: "Jettisons the service module before entering the atmosphere."
}
type "ProvideIndividualLifesupport" {
    description: "Provide life support for crew member on the lunar surface outside the lander."
}
type "ExecuteQuarantine" {
    description: "Implements biological isolation protocols for the crew."
}
type "ExecuteAscentBurn" {
    description: "Fires the ascent engine to lift off from the Moon."
}
type "ExecuteTransearthBurn" {
    description: "Performs the major propulsive maneuver to leave lunar orbit for Earth."
}
type "PropelledSpacecraft" extends "FuelledComponent" {
}
type "Stakeholder" {
}
type "ManageCrewSchedules" {
    description: "Manages the schedule for crew work, sleep, and meals."
}
type "MannedSpaceMission" extends "Mission" {
}
type "ConnectSpacecraft" {
    description: "Performs the physical docking of the two vehicles."
}
type "MechanicallyLatchModules" {
    description: "Engages the docking probe and latches to form a rigid connection between two modules."
}
type "PrepareLanderForSeparation" {
    description: "Activates and configures the Lunar Module for independent flight."
}
type "RetrieveVehicleAndCrew" {
    description: "Locates and recovers the crew and capsule from the ocean."
}
type "ExecuteDescentBurn" {
    description: "Manages the main engine burn for powered descent."
}
type "ObserveAscentDataFromCockpit" {
    description: "Allows the crew to monitor spacecraft displays and provides manual inputs or abort commands if necessary."
}
type "CalculateCourseCorrection" {
    description: "Processes navigational data to determine the required vector for a mid-course correction burn."
}
type "ControlTranslunarInjectionBurn" {
    description: "Guides the TLI burn to ensure accuracy."
}
type "CalculateRendezvousManeuvers" {
    description: "Processes tracking data to compute the required burns for rendezvous."
}
type "David Scott" {
    description: "David Scott flew on Gemini 8 and Apollo 9, and was the Commander of Apollo 15. As the seventh person to walk on the Moon, he was the first to drive the Lunar Roving Vehicle on the lunar surface."
    dateOfBirth: "new Iso8601DateTime(\"1932-06-06\")"
}
type "MonitorOnboardSystems" {
    description: "Allows the crew to observe and manage spacecraft systems from inside the cabin."
}
type "PerformPropellantLoading" {
    description: "Fills and conditions all necessary propellants and consumables for the mission."
}
type "PrepareForExtravehicularActivity" {
    description: "Prepares the cabin and suits for surface exploration."
}
type "GatherLunarSamples" {
    description: "Collects and stores geological samples."
}
type "FuelledComponent" {
    mass_expression: "dryMass + propellantMass"
}
type "GovernmentAgency" extends "Stakeholder" {
}
type "AnalyzeLaunchTelemetryFromGround" {
    description: "Receives and analyzes telemetry on the ground to provide go/no-go decisions to the crew."
}
type "ProvideTranslunarInjectionThrust" {
    description: "Re-ignites the engine to propel the spacecraft out of Earth orbit and toward the Moon."
}
type "ProvideStage1Thrust" {
    description: "Provides primary thrust from liftoff through the first two and a half minutes of flight."
}
type "SecureReturnedSamples" {
    description: "Secures the collected lunar materials for scientific study."
}
type "PerformPostLandingCheck" {
    description: "Verifies the state of the vehicle after landing on the Moon."
}
type "ExecuteRendezvousBurns" {
    description: "Fires RCS thrusters to execute the rendezvous plan and approach the target vehicle."
}
type "Eugene Cernan" {
    description: "Eugene \"Gene\" Cernan was one of three men to fly to the Moon twice, on Apollo 10 and Apollo 17. As Commander of Apollo 17, he was the last person to walk on the Moon."
    dateOfBirth: "new Iso8601DateTime(\"1934-03-14\")"
    dateOfDeath: "new Iso8601DateTime(\"2017-01-16\")"
}
type "TrackAndRelayGroundSignals" {
    description: "Manages the global network of ground stations to maintain a continuous link with the spacecraft."
}
type "Public" extends "Stakeholder" {
}
type "J-2" extends "RocketEngine" {
    description: "A liquid-fuel cryogenic rocket engine used in the S-II and S-IVB stages, notable for its restart capability."
    thrustVacuum: 1033 kN
    specificImpulseVacuum: 421 s
}
type "TransferCrewAndMaterials" {
    description: "Moves the crew and collected samples from the ascent stage to the main vehicle."
}
type "PerformSplashdown" {
    description: "Manages the final water landing."
}
type "MonitorDownlinkedTelemetry" {
    description: "Allows ground controllers to observe spacecraft systems via telemetry."
}
type "PerformCrewIngress" {
    description: "Transfers the flight crew into the vehicle and secures them for launch."
}
type "TransmitAndReceiveSpacecraftSignals" {
    description: "Manages the vehicle's antennas and radios to send and receive data."
}
type "ProvideOrbitInsertionThrust" {
    description: "Provides the final thrust needed to achieve a stable Earth orbit."
}
type "GuideAtmosphericEntry" {
    description: "Controls the vehicle's trajectory through the atmosphere to manage heat and aim for the landing zone."
}
type "PerformOrbitalSystemCheckouts" {
    description: "Performs system health checks while in lunar orbit."
}
type "CompleteExtravehicularActivity" {
    description: "Concludes the surface exploration and has the crew re-enter the vehicle."
}
type "DiscardAscentStage" {
    description: "Jettisons the Lunar Module ascent stage."
}
type "Mission" {
}
type "VehicleAssemblyBuilding" {
    description: "The massive facility used to stack the space vehicle."
}
type "ExecuteTerminalCountdown" {
    description: "Executes the final automated and manual checks leading to launch."
}

requirements {
    requirement "performLunarMissionSpecification" {
        description: "Apollo 11 Function requirements group"
    }
    requirement "apollo11MissionSpecification" {
        description: "Apollo 11 Mission requirements group"
    }
    requirement "apollo11MissionSystemSpecification" {
        description: "Apollo 11 System requirements group"
    }
    requirement "FLR-R001" {
        description: "The system shall enable the loading of all required propellants and consumables into the vehicle's storage systems."
        title: "PropellantLoadingRequirement"
    }
    requirement "FLR-R002" {
        description: "The system shall maintain all loaded propellants and consumables at their required temperature and pressure prior to launch."
        title: "ConsumableConditioningRequirement"
    }
    requirement "FLR-R003" {
        description: "The system shall provide a safe method for the flight crew to enter and be secured within the crew cabin."
        title: "CrewIngressRequirement"
    }
    requirement "FLR-R004" {
        description: "The system shall verify the pressure integrity of the crew cabin after crew ingress."
        title: "CabinPressurizationRequirement"
    }
    requirement "FLR-R005" {
        description: "The system shall execute a synchronized sequence of pre-launch checks and configurations."
        title: "CountdownSequencingRequirement"
    }
    requirement "FLR-R006" {
        description: "The system shall be capable of holding or aborting the countdown if any critical system parameter violates pre-defined limits."
        title: "CountdownAbortRequirement"
    }
    requirement "FLR-R007" {
        description: "The system shall initiate first stage main propulsion upon receiving a valid, authenticated launch command."
        title: "Stage1ThrustInitiationRequirement"
    }
    requirement "FLR-R008" {
        description: "The system shall generate a propulsive force from the first stage that results in a positive vertical acceleration at liftoff."
        title: "Stage1ThrustMagnitudeRequirement"
    }
    requirement "FLR-R009" {
        description: "The system shall initiate second stage propulsion following the clean separation of the first stage."
        title: "Stage2ThrustInitiationRequirement"
    }
    requirement "FLR-R010" {
        description: "The system shall provide continuous thrust from the second stage to propel the vehicle through the upper atmosphere."
        title: "Stage2PropulsionRequirement"
    }
    requirement "FLR-R011" {
        description: "The system shall initiate third stage propulsion to provide the final thrust needed to achieve a stable Earth orbit."
        title: "OrbitInsertionThrustInitiationRequirement"
    }
    requirement "FLR-R012" {
        description: "The system shall terminate third stage thrust upon receiving a cutoff command."
        title: "OrbitInsertionThrustTerminationRequirement"
    }
    requirement "FLR-R013" {
        description: "The system shall autonomously control the vehicle's trajectory during ascent according to a pre-programmed flight plan."
        title: "AscentGuidanceRequirement"
    }
    requirement "FLR-R014" {
        description: "The system shall issue commands to separate expended stages at pre-determined points in the ascent profile."
        title: "AscentStagingCommandRequirement"
    }
    requirement "FLR-R015" {
        description: "The system shall display critical vehicle parameters to the flight crew during ascent."
        title: "CrewAscentMonitoringRequirement"
    }
    requirement "FLR-R016" {
        description: "The system shall provide an interface for the crew to manually initiate a launch abort sequence."
        title: "CrewAscentAbortRequirement"
    }
    requirement "FLR-R017" {
        description: "The system shall receive, process, and display real-time telemetry from the launch vehicle to ground personnel."
        title: "GroundTelemetryAnalysisRequirement"
    }
    requirement "FLR-R018" {
        description: "The system shall compare vehicle performance against pre-defined limits and provide go/no-go recommendations."
        title: "GroundPerformanceVerificationRequirement"
    }
    requirement "FLR-R019" {
        description: "The system shall provide a propulsive force to cause the vehicle to escape Earth's orbit."
        title: "TliThrustInitiationRequirement"
    }
    requirement "FLR-R020" {
        description: "The system shall guide the translunar injection burn to achieve the required trajectory accuracy."
        title: "TliThrustControlRequirement"
    }
    requirement "FLR-R021" {
        description: "The system shall enable the primary spacecraft to separate from its launch vehicle adapter."
        title: "ModuleSeparationRequirement"
    }
    requirement "FLR-R022" {
        description: "The system shall enable the controlled repositioning of the primary spacecraft after separation."
        title: "ModuleRepositioningRequirement"
    }
    requirement "FLR-R023" {
        description: "The system shall enable the physical connection and sealing of two separate spacecraft modules in space."
        title: "ModuleConnectionRequirement"
    }
    requirement "FLR-R024" {
        description: "The system shall provide a propulsive force for major in-space trajectory changes."
        title: "InSpacePropulsionRequirement"
    }
    requirement "FLR-R025" {
        description: "The system shall provide low-thrust propulsive forces to control the orientation of the spacecraft."
        title: "AttitudeControlRequirement"
    }
    requirement "FLR-R026" {
        description: "The system shall enable the acquisition of celestial navigation data."
        title: "NavigationalDataAcquisitionRequirement"
    }
    requirement "FLR-R027" {
        description: "The system shall calculate the vehicle's current trajectory based on navigational data."
        title: "TrajectoryCalculationRequirement"
    }
    requirement "FLR-R028" {
        description: "The system shall calculate the required propulsive maneuver to correct any deviations from the planned trajectory."
        title: "CourseCorrectionCalculationRequirement"
    }
    requirement "FLR-R029" {
        description: "The system shall execute a calculated course correction maneuver."
        title: "CourseCorrectionExecutionRequirement"
    }
    requirement "FLR-R030" {
        description: "The system shall display the status of all critical subsystems to the crew."
        title: "CrewSystemMonitoringRequirement"
    }
    requirement "FLR-R031" {
        description: "The system shall transmit the status of all critical subsystems to the ground."
        title: "GroundSystemMonitoringRequirement"
    }
    requirement "FLR-R032" {
        description: "The system shall transmit voice, telemetry, and video data to the ground network."
        title: "SpacecraftDataTransmissionRequirement"
    }
    requirement "FLR-R033" {
        description: "The system shall receive and process commands and voice data from the ground network."
        title: "SpacecraftDataReceptionRequirement"
    }
    requirement "FLR-R034" {
        description: "The system shall track the spacecraft's position and velocity from Earth."
        title: "GroundTrackingRequirement"
    }
    requirement "FLR-R035" {
        description: "The system shall relay communications between the spacecraft and Mission Control."
        title: "GroundCommunicationRelayRequirement"
    }
    requirement "FLR-R036" {
        description: "The system shall provide a structured timeline for all crew activities, including work, meals, and sleep."
        title: "CrewActivitySchedulingRequirement"
    }
    requirement "FLR-R037" {
        description: "The system shall provide an environment that allows for crew sleep."
        title: "CrewRestProvisionRequirement"
    }
    requirement "FLR-R038" {
        description: "The system shall provide a retrograde propulsive force sufficient to reduce the vehicle's velocity for lunar capture."
        title: "LoiThrustRequirement"
    }
    requirement "FLR-R039" {
        description: "The system shall guide the lunar orbit insertion burn to achieve the target orbital parameters."
        title: "LoiGuidanceRequirement"
    }
    requirement "FLR-R040" {
        description: "The system shall enable the crew to activate and verify the health of all Lunar Module systems prior to separation."
        title: "LanderSystemVerificationRequirement"
    }
    requirement "FLR-R041" {
        description: "The system shall confirm all interfaces between the lander and the main vehicle are ready for separation."
        title: "LanderInterfaceVerificationRequirement"
    }
    requirement "FLR-R042" {
        description: "The system shall allow the crew to transfer from the primary vehicle to the landing vehicle."
        title: "LanderCrewTransferRequirement"
    }
    requirement "FLR-R043" {
        description: "The system shall allow the crew to configure the landing vehicle for independent flight."
        title: "LanderConfigurationRequirement"
    }
    requirement "FLR-R044" {
        description: "The system shall disengage all docking latches connecting two vehicles."
        title: "ModuleDisengagementRequirement"
    }
    requirement "FLR-R045" {
        description: "The system shall provide a controlled, low-velocity maneuver to create distance between two separated vehicles."
        title: "ModuleSeparationManeuverRequirement"
    }
    requirement "FLR-R046" {
        description: "The system shall provide a continuous, throttleable propulsive force to control the rate of descent to the lunar surface."
        title: "DescentPropulsionRequirement"
    }
    requirement "FLR-R047" {
        description: "The system shall provide real-time altitude and velocity data to the crew and ground controllers."
        title: "DescentDataDisplayRequirement"
    }
    requirement "FLR-R048" {
        description: "The system shall process sensor data to provide accurate navigation information during landing."
        title: "DescentDataProcessingRequirement"
    }
    requirement "FLR-R049" {
        description: "The system shall allow the crew to take manual control of the lander's attitude."
        title: "ManualAttitudeControlRequirement"
    }
    requirement "FLR-R050" {
        description: "The system shall enable horizontal repositioning to avoid hazards."
        title: "ManualPositionControlRequirement"
    }
    requirement "FLR-R051" {
        description: "The system shall allow the crew to place all lander systems into a safe, quiescent state after touchdown."
        title: "PostLandingSafingRequirement"
    }
    requirement "FLR-R052" {
        description: "The system shall allow the crew to verify that the vehicle is in a stable state and ready for surface operations."
        title: "PostLandingVerificationRequirement"
    }
    requirement "FLR-R053" {
        description: "The system shall allow the crew to don and check out their spacesuits."
        title: "EgressPreparationRequirement"
    }
    requirement "FLR-R054" {
        description: "The system shall be capable of depressurizing the lander cabin to the external vacuum environment."
        title: "CabinDepressurizationRequirement"
    }
    requirement "FLR-R055" {
        description: "The system shall enable astronauts to traverse the lunar surface."
        title: "SurfaceMobilityRequirement"
    }
    requirement "FLR-R056" {
        description: "The system shall enable the crew to deploy a national symbol on the lunar surface."
        title: "SymbolicDeploymentRequirement"
    }
    requirement "FLR-R057" {
        description: "The system shall enable the collection of lunar material."
        title: "SampleCollectionRequirement"
    }
    requirement "FLR-R058" {
        description: "The system shall provide storage for collected samples that prevents terrestrial contamination."
        title: "SampleStorageRequirement"
    }
    requirement "FLR-R059" {
        description: "The system shall enable the crew to deploy scientific experiment packages on the lunar surface."
        title: "InstrumentDeploymentRequirement"
    }
    requirement "FLR-R060" {
        description: "The system shall enable the deployment of a visual transmission system."
        title: "VisualTransmissionRequirement"
    }
    requirement "FLR-R061" {
        description: "The system shall allow the crew to re-enter the lander cabin with all collected samples and equipment."
        title: "EvaIngressRequirement"
    }
    requirement "FLR-R062" {
        description: "The system shall allow the crew to re-pressurize the lander cabin."
        title: "CabinRepressurizationRequirement"
    }
    requirement "FLR-R063" {
        description: "The system shall provide a habitable environment for the crew to sleep and eat on the lunar surface."
        title: "SurfaceHabitationRequirement"
    }
    requirement "FLR-R064" {
        description: "The system shall manage the timeline to ensure readiness for ascent within the required orbital mechanics window."
        title: "SurfaceTimelineManagementRequirement"
    }
    requirement "FLR-R065" {
        description: "The system shall initiate the ascent engine using the descent stage as a launch platform."
        title: "AscentThrustInitiationRequirement"
    }
    requirement "FLR-R066" {
        description: "The system shall provide a continuous, non-throttleable propulsive force to achieve lunar orbit."
        title: "AscentPropulsionRequirement"
    }
    requirement "FLR-R067" {
        description: "The system shall process tracking data to compute the required maneuvers for rendezvous."
        title: "RendezvousCalculationRequirement"
    }
    requirement "FLR-R068" {
        description: "The system shall execute a series of precise burns to match the orbit of and approach the target vehicle."
        title: "RendezvousExecutionRequirement"
    }
    requirement "FLR-R069" {
        description: "The system shall allow the crew to pilot the final approach to the target vehicle."
        title: "DockingApproachRequirement"
    }
    requirement "FLR-R070" {
        description: "The system shall achieve a successful capture and rigid latching of the two vehicles."
        title: "DockingLatchingRequirement"
    }
    requirement "FLR-R071" {
        description: "The system shall allow the crew to move between two docked vehicles."
        title: "IntervehicularCrewTransferRequirement"
    }
    requirement "FLR-R072" {
        description: "The system shall allow the crew to transfer all collected lunar samples to the return vehicle."
        title: "IntervehicularSampleTransferRequirement"
    }
    requirement "FLR-R073" {
        description: "The system shall allow the crew to seal the hatches and disengage the docking latches."
        title: "AscentStageSeparationRequirement"
    }
    requirement "FLR-R074" {
        description: "The system shall provide a controlled separation from the ascent stage."
        title: "AscentStageDisposalRequirement"
    }
    requirement "FLR-R075" {
        description: "The system shall provide a propulsive force to cause the vehicle to escape lunar orbit and enter a trans-Earth trajectory."
        title: "TeiPropulsionRequirement"
    }
    requirement "FLR-R076" {
        description: "The system shall allow the crew to perform final system checks and configure the vehicle for atmospheric entry."
        title: "ReentryConfigurationRequirement"
    }
    requirement "FLR-R077" {
        description: "The system shall execute a pyrotechnic separation of the primary crew vehicle from its main support module."
        title: "ServiceModuleJettisonRequirement"
    }
    requirement "FLR-R078" {
        description: "The system shall ensure the two separated modules do not re-contact."
        title: "CleanSeparationRequirement"
    }
    requirement "FLR-R079" {
        description: "The system shall use the vehicle's shape and orientation to control its trajectory through the atmosphere."
        title: "ReentryGuidanceRequirement"
    }
    requirement "FLR-R080" {
        description: "The system shall protect the crew and vehicle structure from the extreme heat of atmospheric entry."
        title: "HeatDissipationRequirement"
    }
    requirement "FLR-R081" {
        description: "The system shall deploy a sequence of aerodynamic decelerators at pre-defined altitudes."
        title: "DeceleratorDeploymentRequirement"
    }
    requirement "FLR-R082" {
        description: "The system shall slow the vehicle to a safe landing velocity."
        title: "FinalVelocityRequirement"
    }
    requirement "FLR-R083" {
        description: "The system shall execute the final water landing."
        title: "WaterLandingRequirement"
    }
    requirement "FLR-R084" {
        description: "The system shall automatically deploy flotation devices and beacons after landing."
        title: "PostLandingSystemsRequirement"
    }
    requirement "FLR-R085" {
        description: "The system shall allow recovery forces to locate, secure, and retrieve the crew and crew vehicle from the ocean."
        title: "CrewRecoveryRequirement"
    }
    requirement "FLR-R086" {
        description: "The system shall transport the crew and crew vehicle to the primary recovery ship."
        title: "VesselTransferRequirement"
    }
    requirement "FLR-R087" {
        description: "The system shall implement Level V biological quarantine procedures for the crew upon recovery."
        title: "QuarantineImplementationRequirement"
    }
    requirement "FLR-R088" {
        description: "The system shall place the crew into a specialized quarantine facility for a 21-day period."
        title: "QuarantineHabitationRequirement"
    }
    requirement "FLR-R089" {
        description: "The system shall place the returned lunar samples into secure, contamination-free containers."
        title: "SamplePreservationRequirement"
    }
    requirement "FLR-R090" {
        description: "The system shall transport the samples to a designated receiving laboratory for analysis."
        title: "SampleTransportRequirement"
    }
    requirement "FLR-R091" {
        description: "The Apollo 11 Mission's navigation system shall determine spacecraft position with an accuracy of +/- 5 kilometers\n\t\t\tduring Trans-Lunar Coast, at least once every 6 hours."
        title: "TGNCNavigationAccuracy"
    }
    requirement "FLR-R092" {
        description: "The Apollo 11 Mission's control system shall enable manual piloting of the lunar lander to allow for\n\t\t\thorizontal repositioning of +/- 200 meters within the landing ellipse to avoid hazards."
        title: "ManualPilotingLogic"
    }
    requirement "FLR-R093" {
        description: "All major system interfaces of the Apollo 11 Mission (e.g., spacecraft stacking, docking mechanisms)\n\t\t\tshall demonstrate 100% compatibility through full-scale integrated fit checks and electrical interface tests\n\t\t\tprior to final assembly."
        title: "SystemInterfaceCompatibility"
    }
    requirement "FLR-R094" {
        description: "All flight crew members for the Apollo 11 Mission shall complete a minimum of 200 hours\n\t\t\tof specialized lunar lander simulator training prior to launch."
        title: "TrainingSimulatorHours"
    }
    requirement "FLR-R095" {
        description: "The guidance system shall calculate and execute trajectory adjustments \n               to maintain the designated mission corridor."
        title: "CourseCorrectionCapability"
    }
    requirement "FLR-R096" {
        description: "The system shall autonomously maintain the commanded orientation \n               for thermal regulation and communications."
        title: "SpacecraftOrientationControl"
    }
    requirement "FLR-R097" {
        description: "The landing system shall acquire real-time altitude and velocity telemetry \n               during the powered descent phase."
        title: "DescentDataAcquisition"
    }
    requirement "FLR-R098" {
        description: "The launch system shall provide sufficient total impulse to overcome \n               Earth's gravity and achieve orbit insertion."
        title: "LaunchVehicleAscentThrust"
    }
    requirement "FLR-R099" {
        description: "The lander shall provide the capability to vary engine thrust to control \n               the rate of descent to the lunar surface."
        title: "DescentRateModulation"
    }
    requirement "FLR-R100" {
        description: "The entry vehicle shall dissipate aerodynamic heat to protect \n               internal volumes during atmospheric return."
        title: "ThermalProtectionLogic"
    }
    requirement "FLR-R101" {
        description: "The landing gear system shall mechanically attenuate landing forces \n               to prevent structural damage to the spacecraft."
        title: "TouchdownEnergyDissipation"
    }
    requirement "FLR-R102" {
        description: "The life support system shall remove metabolic byproducts and \n               particles from the cabin air."
        title: "AtmosphericScrubbingFunction"
    }
    requirement "FLR-R103" {
        description: "The communication system shall ensure the fidelity of transmitted \n               data between the spacecraft and ground segment."
        title: "TelemetryStreamIntegrity"
    }
    requirement "FLR-R104" {
        description: "The Extravehicular Mobility Unit shall provide a constant pressurized \n               volume for the astronaut in a vacuum environment."
        title: "EMUPressureMaintenance"
    }
    requirement "FLR-R105" {
        description: "The flight software shall be verified to perform all guidance \n               and control functions without mission-critical errors."
        title: "SoftwareMissionProficiency"
    }
    requirement "FLR-R106" {
        description: "Spacecraft materials shall maintain their structural and functional \n               properties under vacuum and thermal cycling."
        title: "SpaceEnvironmentResilience"
    }
    requirement "FLR-R107" {
        description: "The system shall capture high-resolution still and motion imagery of \n               lunar operations."
        title: "VisualDataCapture"
    }
    requirement "FLR-R108" {
        description: "The life support system shall provide localized collection and \n               sealed storage for all metabolic waste."
        title: "WasteContainmentFunction"
    }
    requirement "FLR-R109" {
        description: "The communication system shall establish a dedicated VHF audio link \n               between the LM and CSM during lunar operations."
        title: "CrossVehicleAudioLink"
    }
    requirement "FLR-R110" {
        description: "Critical systems shall maintain nominal output levels despite \n               the loss of any single primary component."
        title: "ComponentFaultTolerance"
    }
    requirement "FLR-R111" {
        description: "The system shall provide medical kits and diagnostic guides \n               enabling the crew to treat minor injuries."
        title: "HumanMedicalSelfSufficiency"
    }
    requirement "FLR-R112" {
        description: "The ground segment shall mirror all incoming telemetry to \n               multiple geographic locations for permanent archival."
        title: "DataRedundancyArchiving"
    }
    requirement "HLR-R001" {
        description: "The Apollo 11 Mission shall safely return all three crew members to Earth,\n\t\t\twith no life-threatening injuries or long-term health impacts, by July 24, 1969."
        title: "CrewReturnSafetyRequirement"
        maxAcceptableInjuries: 0
        require: ["crewInjuryCount <= maxAcceptableInjuries"]
    }
    requirement "HLR-R002" {
        description: "The Apollo 11 Mission shall achieve a soft landing of the crewed lunar lander on the lunar surface\n\t\t\twithin the designated landing zone (Tranquility Base), by July 20, 1969."
        title: "LunarLanderSoftLandingRequirement"
        maxVerticalVelocity: 2 m/s
        maxHorizontalVelocity: 0.5 m/s
        maxLandingZoneDeviation: 1000 m
        require: ["actualVerticalVelocity <= maxVerticalVelocity and actualHorizontalVelocity <= maxHorizontalVelocity and actualLandingZoneDeviation <= maxLandingZoneDeviation"]
    }
    requirement "HLR-R003" {
        description: "The Apollo 11 Mission launch system shall successfully place the spacecraft into a Trans-Lunar Injection (TLI) trajectory\n\t\t\twith a delta-V accuracy of +/- 0.5 meters per second and a path leading to lunar orbit insertion within 75 hours from launch,\n\t\t\ton July 16, 1969."
        title: "TransLunarInjectionAccuracyRequirement"
        maxDeltaVAccuracy: 0.5 m/s
        maxTimeToLunarOrbitInsertion: 75 h
        require: ["actualDeltaVAccuracy <= maxDeltaVAccuracy and actualTimeToLunarOrbitInsertion <= maxTimeToLunarOrbitInsertion"]
    }
    requirement "HLR-R004" {
        description: "The Apollo 11 Mission's communication system shall maintain continuous voice and telemetry contact with ground control\n\t\t\tfor >= 95% of the mission duration, with no single blackout exceeding 10 minutes during lunar farside passes,\n\t\t\tthroughout the 8-day mission."
        title: "CommunicationUptimeRequirement"
        minCommunicationUptimePercentage: 95 percent
        allowedSingleBlackoutDuration: 10 min
        require: ["actualCommunicationUptimePercentage >= minCommunicationUptimePercentage and maxSingleBlackoutDuration <= allowedSingleBlackoutDuration"]
    }
    requirement "HLR-R005" {
        description: "The spacecraft system shall maintain an internal environment that supports crew health and cognitive performance for the mission duration."
        title: "SustainedCabinHabitability"
    }
    requirement "HLR-R006" {
        description: "The Apollo 11 Mission shall collect a minimum of 20 kilograms of documented lunar surface material,\n\t\t\tincluding contingency and bulk samples, and return them to Earth within the Mobile Quarantine Facility,\n\t\t\tby July 24, 1969."
        title: "LunarSampleCollectionQuantityRequirement"
        minRequiredSampleMass: 20 kg
        maxContaminationPercentage: 0.1 percent
        require: ["actualSampleMass >= minRequiredSampleMass and contaminationPercentage <= maxContaminationPercentage"]
    }
    requirement "HLR-R007" {
        description: "The Apollo 11 Mission shall deploy a visual transmission system and at least two scientific experiment packages\n\t\t\ton the lunar surface, and these packages shall transmit data to Earth for a minimum of 1 month post-landing."
        title: "ScientificInstrumentDeploymentRequirement"
        minRequiredVisualSystems: 1
        minRequiredSciencePackages: 2
        minDataTransmissionDuration: "30 d"
        require: ["actualDeployedVisualSystems >= minRequiredVisualSystems and actualDeployedSciencePackages >= minRequiredSciencePackages and actualDataTransmissionDuration >= minDataTransmissionDuration"]
    }
    requirement "HLR-R008" {
        description: "The Apollo 11 Mission shall capture and return to Earth at least 1,000 high-resolution still photographs\n\t\t\tand at least 30 minutes of motion picture footage documenting the lunar terrain, deployed equipment,\n\t\t\tand crew activities, by July 24, 1969."
        title: "LunarVisualDocumentationRequirement"
        minStillPhotosCount: 1000
        minMotionPictureDuration: 30 min
        require: ["actualStillPhotosCount >= minStillPhotosCount and actualMotionPictureDuration >= minMotionPictureDuration"]
    }
    requirement "HLR-R009" {
        description: "The Apollo 11 Mission shall successfully plant a national symbol on the lunar surface,\n\t\t\tensuring it remains upright and visible for subsequent observation (if possible),\n\t\t\tduring the initial Extravehicular Activity on July 20, 1969."
        title: "NationalSymbolDeploymentRequirement"
        require: ["symbolSuccessfullyPlanted and symbolRemainsUpright and symbolVisibility"]
    }
    requirement "HLR-R010" {
        description: "The Apollo 11 Mission shall implement and adhere to Level V biological quarantine procedures\n\t\t\tfor the crew and extraterrestrial samples upon return, ensuring no detectable extraterrestrial\n\t\t\tbiological contamination is released into Earth's biosphere, throughout the 21-day quarantine period."
        title: "PlanetaryProtectionQuarantineRequirement"
        requiredQuarantineLevel: 5
        require: ["actualQuarantineLevel == requiredQuarantineLevel and not biologicalContaminationDetected and not contaminationThresholdExceeded"]
    }
    requirement "HLR-R011" {
        description: "The mission shall maintain the capability to determine spacecraft position with sufficient \n               accuracy to ensure a safe lunar arrival."
        title: "TrajectoryNavigationPrecision"
    }
    requirement "HLR-R026" {
        description: "The Apollo 11 Mission's waste management system shall effectively collect and store all solid and liquid waste\n\t\t\tfor the entire mission, preventing contamination of the cabin environment."
        title: "WasteManagementEffectiveness"
        require: ["not wasteContaminationDetected"]
    }
    requirement "HLR-R028" {
        description: "The Apollo 11 Mission's crew's sleep environment shall allow for at least 6 hours of undisturbed sleep per 24-hour period,\n\t\t\tcharacterized by noise levels below 60 decibels."
        title: "CrewSleepEnvironmentQuality"
        minUndisturbedSleepDuration: 6 h
        maxNoiseLevelReqd: "60 dB"
        require: ["actualUndisturbedSleepDuration >= minUndisturbedSleepDuration and actualNoiseLevelMax <= maxNoiseLevelReqd"]
    }
    requirement "HLR-R030" {
        description: "The Apollo 11 Mission's crew interface system shall provide visual and auditory alarms for all critical system failures,\n\t\t\twith an alarm response time of <= 1 second."
        title: "CrewAlarmResponseTime"
        maxAlarmResponseTime: 1 s
        require: ["actualAlarmResponseTime <= maxAlarmResponseTime"]
    }
    requirement "HLR-R032" {
        description: "The Apollo 11 Mission's lunar lander's communication system shall enable voice communication between the crew on the surface\n\t\t\tand the crew in lunar orbit, with a signal clarity of 90% (subjective scoring)."
        title: "LunarLanderToOrbiterVoiceComm"
        minSignalClarity: 90 percent
        require: ["actualSignalClarity >= minSignalClarity"]
    }
    requirement "HLR-R033" {
        description: "The Apollo 11 Mission's Extravehicular Mobility Unit shall provide life support for a minimum of 4 hours\n\t\t\tof continuous Extravehicular Activity in the lunar environment."
        title: "ExtravehicularMobilityUnitLifeSupportDuration"
        minEMULifeSupportDuration: 4 h
        require: ["actualEMULifeSupportDuration >= minEMULifeSupportDuration"]
    }
    requirement "HLR-R035" {
        description: "The Apollo 11 Mission's Extravehicular Mobility Unit shall allow for manual dexterity sufficient to operate small tools and manipulate samples\n\t\t\twith gloved hands, demonstrating a 90% success rate for task completion during Extravehicular Activity simulations."
        title: "ExtravehicularMobilityUnitDexterityForTasks"
        minTaskCompletionSuccessRate: 90 percent
        require: ["actualTaskCompletionSuccessRate >= minTaskCompletionSuccessRate"]
    }
    requirement "HLR-R036" {
        description: "The Apollo 11 Mission's lunar surface mobility system shall enable astronauts to traverse distances of up to 50 meters\n\t\t\tfrom the lander during Extravehicular Activity with less than 10% fatigue impact on subsequent tasks."
        title: "LunarSurfaceMobilityRange"
        minTraverseDistanceCapability: 50 m
        maxFatigueImpact: 10 percent
        require: ["actualTraverseDistanceCapability >= minTraverseDistanceCapability and actualFatigueImpact <= maxFatigueImpact"]
    }
    requirement "HLR-R037" {
        description: "All Extravehicular Activity tools used in the Apollo 11 Mission shall be operable with a minimum of two points of contact (e.g., tethered or hand-held)\n\t\t\tto prevent loss in lunar gravity."
        title: "ExtravehicularActivityToolSecurity"
        maxToolsLost: 0
        require: ["toolsMeetContactRequirement and toolsLostDuringEVA <= maxToolsLost"]
    }
    requirement "HLR-R038" {
        description: "Apollo 11 Mission's lunar samples shall be collected and stored in a manner that preserves their pristine lunar environment\n\t\t\tand prevents terrestrial contamination, ensuring less than 0.1% terrestrial material contamination by mass."
        title: "SamplePristinePreservation"
        maxTerrestrialContamination: 0.1 percent
        require: ["actualTerrestrialContamination <= maxTerrestrialContamination"]
    }
    requirement "HLR-R040" {
        description: "The Apollo 11 Mission shall obtain geological context for all collected samples through photographic documentation\n\t\t\twith clear scale references, ensuring >= 90% of samples are correlated with their collection location."
        title: "SampleGeologicalContext"
        minSampleCorrelationPercentage: 90 percent
        require: ["actualSampleCorrelationPercentage >= minSampleCorrelationPercentage and photosHaveScaleReferences"]
    }
    requirement "HLR-R045" {
        description: "The Apollo 11 Mission shall have redundancy (N+1 or greater) for all critical single-point-of-failure components\n\t\t\tin propulsion, Guidance, Navigation, and Control, and life support systems, ensuring continued operation after a single failure."
        title: "CriticalComponentRedundancy"
        require: ["not singleFailureCausesOperationalLoss"]
    }
    requirement "HLR-R046" {
        description: "Apollo 11 Mission Control shall be capable of initiating and executing an abort sequence\n\t\t\tfrom any point during ascent or Trans-Lunar/Earth Coast within 5 minutes of an abort decision."
        title: "AbortSequenceExecutionTime"
        maxAbortExecutionTime: 5 min
        require: ["actualAbortExecutionTime <= maxAbortExecutionTime"]
    }
    requirement "HLR-R047" {
        description: "The Apollo 11 Mission's crew shall be able to manually override automated flight control systems\n\t\t\tfor all major maneuvers (e.g., Trans-Lunar Injection, Lunar Orbit Insertion, Powered Descent Initiation) with a response time of <= 1 second."
        title: "ManualOverrideResponseTime"
        maxOverrideResponseTime: 1 s
        require: ["actualOverrideResponseTime <= maxOverrideResponseTime"]
    }
    requirement "HLR-R048" {
        description: "The Apollo 11 Mission's ground segment shall provide up-to-date trajectory and state vector information\n\t\t\tto the flight crew at least every 10 minutes during dynamic flight phases."
        title: "GroundSegmentDataUpdateFrequency"
        maxUpdateInterval: 10 min
        require: ["actualUpdateInterval <= maxUpdateInterval"]
    }
    requirement "HLR-R049" {
        description: "The Apollo 11 Mission's lunar lander's ascent stage shall be capable of launching from the lunar surface\n\t\t\twithin 24 hours of landing, to align with rendezvous opportunities in lunar orbit."
        title: "LunarLanderAscentReadinessTime"
        maxTimeToAscentLaunch: 24 h
        require: ["actualTimeFromLandingToAscentLaunch <= maxTimeToAscentLaunch"]
    }
    requirement "HLR-R050" {
        description: "Apollo 11 Mission Control shall be able to reconfigure and uplink flight software updates\n\t\t\tto the spacecraft's primary computers within 1 hour in response to critical in-flight anomalies."
        title: "FlightSoftwareUpdateTime"
        maxSoftwareUpdateDuration: 1 h
        require: ["actualSoftwareUpdateDuration <= maxSoftwareUpdateDuration"]
    }
    requirement "HLR-R052" {
        description: "The Apollo 11 Mission crew shall demonstrate proficiency in emergency medical procedures\n\t\t\tto handle common in-flight illnesses or minor injuries without external assistance for up to 72 hours."
        title: "CrewEmergencyMedicalProficiency"
        minMedicalProficiencyScore: 90
        minSelfSufficiencyDuration: 72 h
        require: ["medicalProficiencyScore >= minMedicalProficiencyScore and externalAssistanceRequiredBeforeTime >= minSelfSufficiencyDuration"]
    }
    requirement "HLR-R053" {
        description: "The Apollo 11 Mission crew shall be capable of performing all critical in-flight procedures (e.g., docking, re-entry preparation)\n\t\t\twith an error rate of <= 0.5% under nominal conditions."
        title: "InFlightProcedureErrorRate"
        maxProcedureErrorRate: 0.5 percent
        require: ["actualProcedureErrorRate <= maxProcedureErrorRate"]
    }
    requirement "HLR-R054" {
        description: "Apollo 11 Mission astronauts shall be able to consume all provided food and water\n\t\t\tthroughout the mission without experiencing significant gastrointestinal issues or nutrient deficiencies."
        title: "CrewNutritionAndHydration"
        require: ["not significantGIIssuesExperienced and not nutrientDeficienciesObserved"]
    }
    requirement "HLR-R055" {
        description: "The Apollo 11 Mission crew shall maintain a positive psychological state as assessed by structured psychological evaluations\n\t\t\tthroughout the mission."
        title: "CrewPsychologicalState"
        minPsychologicalStateScore: 75
        require: ["actualPsychologicalStateScore >= minPsychologicalStateScore"]
    }
    requirement "HLR-R057" {
        description: "The Apollo 11 Mission spacecraft shall be designed to minimize micro-meteoroid penetration risk\n\t\t\tto critical habitable volumes to less than 1 event in 1,000 missions."
        title: "MicrometeoroidPenetrationRisk"
        maxAcceptablePenetrationProbability: 0.001
        require: ["actualPenetrationProbabilityPerMission <= maxAcceptablePenetrationProbability"]
    }
    requirement "HLR-R059" {
        description: "All external mission elements left on the lunar surface by the Apollo 11 Mission shall be designed to withstand lunar thermal cycles\n\t\t\t(approx. +/- 150°C) and lunar dust exposure for a minimum of 3 months while maintaining structural integrity."
        title: "LunarElementEnvironmentalResistance"
        require: ["structuralIntegrityMaintained"]
    }
    requirement "HLR-R060" {
        description: "The Apollo 11 Mission shall ensure post-flight data retention and archiving of all telemetry, voice, video,\n\t\t\tand scientific data, maintaining 99.99% data availability for future analysis for a minimum of 50 years."
        title: "PostFlightDataArchiving"
        minDataAvailabilityPercentage: 99.99 percent
        minArchivalDuration: "50 yr"
        require: ["actualDataAvailabilityPercentage >= minDataAvailabilityPercentage and actualArchivalDuration >= minArchivalDuration"]
    }
    requirement "HLR-R061" {
        description: "The mission's major maneuvers shall be executed using highly reliable propulsive systems \n               to ensure mission success and return."
        title: "PropulsionSystemMissionReliability"
    }
    requirement "HLR-R062" {
        description: "All primary structures shall withstand the flight environment without compromising \n               mission functionality."
        title: "MechanicalLoadResilience"
    }
    requirement "HLR-R063" {
        description: "The mission shall provide a live visual record of lunar surface operations to satisfy \n               public and scientific return goals."
        title: "LiveVisualBroadcast"
    }
    requirement "HLR-R064" {
        description: "Deployed lunar elements shall remain operational post-landing to provide \n               long-term data return."
        title: "ExtendedScientificOperation"
    }
    requirement "HLR-R065" {
        description: "All mission-critical systems shall be designed for maximum uptime to prevent \n               catastrophic mission loss."
        title: "CriticalSystemAvailability"
    }
    requirement "HLR-R066" {
        description: "The Apollo 11 program shall maintain a public affairs office to provide real-time updates and \n\t\t       educational materials to the global public, ensuring a favorable public opinion rating of \n\t\t       at least 60% throughout the mission duration."
        title: "PublicSupportAndAffairsManagement"
    }
    requirement "HLR-R067" {
        description: "The spacecraft electrical power system shall provide a continuous supply of regulated DC power \n\t\t       to all primary bus systems, maintaining a minimum of 25% energy reserve at all times \n\t\t       during the nominal mission profile."
        title: "ElectricalPowerAvailability"
    }
    requirement "HLR-R068" {
        description: "Launch operations shall execute the final countdown and ignition sequence within a ±2.5-second \n\t\t       launch window to ensure the spacecraft meets the required planetary alignment for the \n\t\t       Trans-Lunar Injection."
        title: "LaunchWindowPrecision"
    }
    requirement "HLR-R069" {
        description: "The mission architecture shall provide the crew and ground controllers with 'workaround' \n\t\t       protocols for all non-critical system anomalies, enabling mission continuation despite \n\t\t       minor hardware or software deviations."
        title: "OperationalWorkaroundProtocol"
    }
    requirement "SHN-N001" {
        description: "The mission shall succeed in all its technical and operational objectives."
        title: "MissionSuccess"
    }
    requirement "SHN-N002" {
        description: "The mission shall ensure the safety and survival of the astronauts throughout all phases."
        title: "AstronautSafety"
    }
    requirement "SHN-N003" {
        description: "The mission's execution and public profile shall be managed to maintain continued public and political support and funding."
        title: "ProgrammaticFunding"
    }
    requirement "SHN-N004" {
        description: "The mission shall demonstrate technological superiority and achieve its primary objectives before any competing nation."
        title: "InternationalPrestige"
    }
    requirement "SHN-N005" {
        description: "The mission shall yield significant scientific data, primarily through the return of lunar samples."
        title: "ScientificReturn"
    }
    requirement "SHN-N006" {
        description: "The mission shall prevent the contamination of Earth's biosphere by potential extraterrestrial life forms."
        title: "PlanetaryProtection"
    }
    requirement "SHN-N007" {
        description: "The mission systems and procedures shall ensure the crew can survive all mission phases."
        title: "CrewSurvival"
    }
    requirement "SHN-N008" {
        description: "The mission shall provide the tools, training, and systems necessary for the successful execution of all assigned procedures."
        title: "ProceduralSuccess"
    }
    requirement "SHN-N009" {
        description: "The mission environment and crew selection shall enable the management of extreme stress and pressure to ensure peak performance."
        title: "PerformanceUnderPressure"
    }
    requirement "SHN-N010" {
        description: "The mission training program shall adequately prepare the crew for all nominal and contingency scenarios."
        title: "TrainingFidelity"
    }
    requirement "SHN-N011" {
        description: "The mission systems shall protect the crew from the known and unknown hazards of the lunar environment."
        title: "EnvironmentalHazardProtection"
    }
    requirement "SHN-N012" {
        description: "The mission systems shall provide the necessary data and communication links to enable real-time problem solving from the ground."
        title: "RealtimeProblemSolving"
    }
    requirement "SHN-N013" {
        description: "The mission shall provide accurate and reliable telemetry to support critical ground-based decision making."
        title: "DataAccuracy"
    }
    requirement "SHN-N014" {
        description: "The mission shall enable continuous communication to provide uninterrupted support to the flight crew."
        title: "ContinuousSupport"
    }
    requirement "SHN-N015" {
        description: "The mission shall be executed with a high degree of coordination between all flight and ground control teams."
        title: "OperationalCoordination"
    }
    requirement "SHN-N016" {
        description: "The Lunar Module vehicle shall perform all its functions successfully, including landing, life support, and ascent."
        title: "LunarModulePerformance"
    }
    requirement "SHN-N017" {
        description: "The development and delivery of the Lunar Module shall adhere to the programmatic schedule and budget."
        title: "ProgrammaticAdherence"
    }
    requirement "SHN-N018" {
        description: "The success of the Lunar Module shall enhance the company's reputation as a leading aerospace contractor."
        title: "ReputationalSuccess"
    }
    requirement "SHN-N019" {
        description: "The mission shall achieve its goal of a crewed lunar landing to demonstrate national technological supremacy."
        title: "NationalPrestige"
    }
    requirement "SHN-N020" {
        description: "The mission shall be conducted successfully and safely to bolster public confidence and morale."
        title: "PublicConfidence"
    }
    requirement "SHN-N021" {
        description: "The mission shall produce sufficient scientific and symbolic returns to justify the significant national financial investment."
        title: "InvestmentJustification"
    }
    requirement "SHN-N022" {
        description: "The mission shall demonstrate advanced technological capabilities relevant to national security."
        title: "NationalSecurityDemonstration"
    }
    requirement "CLR-R001" {
        description: "The S-IC stage shall incorporate five F-1 engines."
        title: "SICEngineConfiguration"
    }
    requirement "CLR-R002" {
        description: "The S-IC stage shall be designed to store and deliver RP-1 (Kerosene) and Liquid Oxygen (LOX) to its engines."
        title: "SICPropellantCompatibility"
    }
    requirement "CLR-R003" {
        description: "The S-IC stage shall provide a gimbaling mechanism for its engines to enable thrust vector control."
        title: "SICGimbalCapability"
    }
    requirement "CLR-R026" {
        description: "The S-IC stage shall incorporate a pyrotechnic separation system to cleanly detach from the S-II stage."
        title: "SICSeparationSystem"
    }
    requirement "CLR-R004" {
        description: "The S-II stage shall incorporate five J-2 engines."
        title: "SIIEngineConfiguration"
    }
    requirement "CLR-R005" {
        description: "The S-II stage shall use Liquid Hydrogen (LH2) and Liquid Oxygen (LOX) propellants."
        title: "SIIPropellantType"
    }
    requirement "CLR-R027" {
        description: "The S-II stage shall maintain structural integrity under the loads of its own thrust and the mass of the upper stages."
        title: "SIIStructuralIntegrity"
    }
    requirement "CLR-R006" {
        description: "The S-IVB stage shall incorporate a single J-2 engine."
        title: "SIVBEngineConfiguration"
    }
    requirement "CLR-R007" {
        description: "The S-IVB's J-2 engine shall be capable of restarting in a zero-gravity environment."
        title: "SIVBRestartCapability"
    }
    requirement "CLR-R028" {
        description: "The S-IVB stage shall incorporate an attitude control system to maintain stable orientation during coast phases."
        title: "SIVBAttitudeControlSystem"
    }
    requirement "CLR-R008" {
        description: "The SaturnVInstrumentUnit shall house the primary guidance and navigation computer for the launch vehicle."
        title: "IUGuidanceComputer"
    }
    requirement "CLR-R009" {
        description: "The SaturnVInstrumentUnit shall issue all primary commands for engine ignition, cutoff, and stage separation during ascent."
        title: "IUCommandAuthority"
    }
    requirement "CLR-R029" {
        description: "The SaturnVInstrumentUnit shall gather and transmit vehicle health and status data to ground telemetry systems."
        title: "IUTelemetrySystem"
    }
    requirement "CLR-R010" {
        description: "The ApolloLaunchEscapeSystem shall utilize a high-thrust solid rocket motor."
        title: "LESPropulsionType"
    }
    requirement "CLR-R011" {
        description: "The ApolloLaunchEscapeSystem shall be capable of being safely jettisoned from the spacecraft after the period of maximum atmospheric stress."
        title: "LESJettisonCapability"
    }
    requirement "CLR-R012" {
        description: "The ApolloCommandModule shall provide a pressurized, habitable volume for a crew of three."
        title: "CMHabitableVolume"
    }
    requirement "CLR-R013" {
        description: "The ApolloCommandModule shall incorporate a heat shield capable of protecting the crew and structure from the heat of atmospheric reentry."
        title: "CMHeatShield"
    }
    requirement "CLR-R014" {
        description: "The ApolloCommandModule shall house a system of parachutes to decelerate the vehicle for a safe water landing."
        title: "CMParachuteSystem"
    }
    requirement "CLR-R030" {
        description: "The ApolloCommandModule shall incorporate a probe and drogue docking mechanism."
        title: "CMDockingMechanism"
    }
    requirement "CLR-R052" {
        description: "The Apollo 11 Mission's guidance system shall calculate mid-course correction burns with a delta-V accuracy of +/- 0.1 meters per second,\n\t\t\tensuring the trajectory remains within mission-defined corridors."
        title: "GNCGuidancePrecision"
        maxDeltaVcalcAccuracy: 0.1 m/s
        require: ["actualDeltaVcalcAccuracy <= maxDeltaVcalcAccuracy and trajectoryWithinCorridor"]
    }
    requirement "CLR-R053" {
        description: "The Apollo 11 Mission's control system shall maintain spacecraft attitude within +/- 0.5 degrees of commanded orientation\n\t\t\tduring orbital and coast phases."
        title: "RCSControlStability"
        maxAttitudeDeviation: 0.5 deg
        require: ["actualAttitudeDeviation <= maxAttitudeDeviation"]
    }
    requirement "CLR-R054" {
        description: "The Apollo 11 Mission's lunar lander's navigation system shall provide real-time altitude and velocity data\n\t\t\twith an accuracy of +/- 1 meter for altitude and +/- 0.1 meters per second for velocity during the final 50 meters of powered descent."
        title: "LMDSRadarPrecision"
        maxAltitudeAccuracy: 1 m
        maxVelocityAccuracy: 0.1 m/s
        require: ["actualAltitudeAccuracy <= maxAltitudeAccuracy and actualVelocityAccuracy <= maxVelocityAccuracy"]
    }
    requirement "CLR-R015" {
        description: "The ApolloServiceModule shall provide a restartable main engine for major in-space maneuvers."
        title: "SMMainPropulsion"
    }
    requirement "CLR-R016" {
        description: "The ApolloServiceModule shall provide a reaction control system for attitude control and minor velocity changes."
        title: "SMAttitudeControl"
    }
    requirement "CLR-R017" {
        description: "The ApolloServiceModule shall house fuel cells to generate the primary electrical power for the Command/Service Module."
        title: "SMPowerGeneration"
    }
    requirement "CLR-R031" {
        description: "The ApolloServiceModule shall incorporate radiators to reject waste heat from the spacecraft into space."
        title: "SMThermalControlSystem"
    }
    requirement "CLR-R051" {
        description: "The Apollo 11 Mission's spacecraft environmental control system shall maintain cabin temperature between 20°C and 25°C\n\t\t\tand oxygen partial pressure between 2.5 pounds per square inch and 4.0 pounds per square inch for all 8 days of the mission, for a crew of three."
        title: "CabinEnvironmentalControlRequirement"
        minCabinTemperatureReqd: 20 °C
        maxCabinTemperatureReqd: 25 °C
        minOxygenPressureReqd: "2.5 pound-force per square inch (psi)"
        maxOxygenPressureReqd: "4.0 pound-force per square inch (psi)"
        require: ["actualCabinTemperatureMin >= minCabinTemperatureReqd and actualCabinTemperatureMax <= maxCabinTemperatureReqd and actualOxygenPressureMin >= minOxygenPressureReqd and actualOxygenPressureMax <= maxOxygenPressureReqd"]
    }
    requirement "CLR-R018" {
        description: "The LunarModuleDescentStage shall provide a main engine with a throttleable thrust range."
        title: "LMDSThrottleableEngine"
    }
    requirement "CLR-R019" {
        description: "The LunarModuleDescentStage shall incorporate a landing gear system capable of absorbing the energy of a lunar touchdown."
        title: "LMDSLandingGear"
    }
    requirement "CLR-R032" {
        description: "The LunarModuleDescentStage shall provide a stable, structurally sound platform for the ascent stage to launch from."
        title: "LMDSLaunchPlatform"
    }
    requirement "CLR-R050" {
        description: "The LunarModuleDescentStage shall command thrust to ensure that at the moment of footpad touchdown, the vehicle's velocities do not exceed the following limits:\n\t\t\t* Vertical Velocity: <= 3.05 meters per second (10 feet per second)\n\t\t\t* Horizontal Velocity: <= 1.22 meters per second (4 feet per second)"
        title: "LMDSVelocityLimits"
    }
    requirement "CLR-R020" {
        description: "The LunarModuleAscentStage shall provide a fixed-thrust engine capable of lifting the stage from the lunar surface into orbit."
        title: "LMASAscentEngine"
    }
    requirement "CLR-R021" {
        description: "The LunarModuleAscentStage shall provide a pressurized cabin for a crew of two."
        title: "LMASCrewCabin"
    }
    requirement "CLR-R033" {
        description: "The LunarModuleAscentStage shall incorporate a radar system to track the Command Module during rendezvous."
        title: "LMASRendezvousRadar"
    }
    requirement "CLR-R022" {
        description: "The PressureSuiteAssembly shall maintain a positive internal pressure environment for the astronaut."
        title: "PSAPressureContainment"
    }
    requirement "CLR-R023" {
        description: "The PressureSuiteAssembly shall allow sufficient mobility for the crew to perform required EVA tasks."
        title: "PSAMobility"
    }
    requirement "CLR-R034" {
        description: "The PressureSuiteAssembly shall provide a helmet with a wide, unobstructed field of view and protective visors."
        title: "PSAHelmetVisibility"
    }
    requirement "CLR-R024" {
        description: "The PortableLifeSupportSystem shall contain and supply breathable oxygen to the crew member."
        title: "PLSSOxygenSupply"
    }
    requirement "CLR-R025" {
        description: "The PortableLifeSupportSystem shall remove carbon dioxide and other contaminants from the suit's atmosphere."
        title: "PLSSContaminantRemoval"
    }
    requirement "CLR-R035" {
        description: "The PortableLifeSupportSystem shall circulate cooling fluid to regulate astronaut body temperature."
        title: "PLSSCoolingSystem"
    }
    requirement "CLR-R036" {
        description: "The StagingInterface shall be capable of transferring all structural loads from the upper stage to the lower stage during powered ascent."
        title: "StagingInterfaceLoadTransfer"
    }
    requirement "CLR-R037" {
        description: "The StagingInterface shall ensure a clean separation with no re-contact between stages upon receiving a valid separation command."
        title: "StagingInterfaceCleanSeparation"
    }
    requirement "CLR-R038" {
        description: "The GuidanceInterface shall transmit control commands with a data integrity of 99.999% or greater."
        title: "GuidanceInterfaceCommandIntegrity"
    }
    requirement "CLR-R039" {
        description: "The GuidanceInterface shall provide real-time telemetry from the stage to the controller with a latency of less than 10 milliseconds."
        title: "GuidanceInterfaceTelemetryLatency"
    }
    requirement "CLR-R040" {
        description: "The CSMInterface shall transfer electrical power from the supplier to the consumer with a voltage drop of less than 2%."
        title: "CSMInterfacePowerTransfer"
    }
    requirement "CLR-R041" {
        description: "The CSMInterface shall transport life support consumables with zero leakage."
        title: "CSMInterfaceLifeSupportIntegrity"
    }
    requirement "CLR-R042" {
        description: "The DockingInterface shall form a rigid structural connection capable of withstanding attitude control maneuvers."
        title: "DockingInterfaceStructuralRigidity"
    }
    requirement "CLR-R043" {
        description: "The DockingInterface shall create a pressurized seal between the two vehicle cabins."
        title: "DockingInterfacePressurizedSeal"
    }
    requirement "CLR-R044" {
        description: "The LMStagingInterface shall execute a clean pyrotechnic separation of the Ascent Stage from the Descent Stage."
        title: "LMStagingInterfaceSeparation"
    }
    requirement "CLR-R045" {
        description: "The EVAInterface shall provide secure, quick-disconnect fittings for all fluid and data connections."
        title: "EVAInterfaceSecureConnection"
    }
    requirement "CLR-R046" {
        description: "The interface between the SaturnVInstrumentUnit and the ApolloSpacecraftLMAdapter shall withstand all structural loads during launch and ascent."
        title: "LVPayloadInterfaceStructural"
    }
    requirement "CLR-R047" {
        description: "The ApolloSpacecraftLMAdapter shall deploy the Lunar Module upon command with sufficient force to prevent re-contact."
        title: "SLALMDeployment"
    }
    requirement "CLR-R048" {
        description: "The interface between the ApolloCommandModule and the ApolloLaunchEscapeSystem shall maintain structural integrity under all launch and ascent aerodynamic loads."
        title: "LESInterfaceStructuralIntegrity"
    }
    requirement "CLR-R049" {
        description: "The interface between the ApolloCommandModule and the ApolloLaunchEscapeSystem shall reliably transmit abort and jettison commands to the LES motors."
        title: "LESInterfaceCommandReliability"
    }
    requirement "CLR-R055" {
        description: "The Apollo 11 Mission's launch vehicle's first stage propulsion shall provide a minimum thrust of 34.5 Meganewtons at liftoff."
        title: "SICThrustMagnitude"
        minLiftoffThrust: "34.5 MN"
        require: ["actualLiftoffThrust >= minLiftoffThrust"]
    }
    requirement "CLR-R056" {
        description: "The Apollo 11 Mission's in-space propulsion system shall be capable of performing at least 3 full-duration major burns\n\t\t\t(e.g., Lunar Orbit Insertion, Trans-Earth Injection) with a 99.8% ignition success rate for each burn."
        title: "SPSIgnitionReliability"
        minMajorBurnCount: 3
        minIgnitionSuccessRate: 99.8 percent
        require: ["actualMajorBurnCount >= minMajorBurnCount and actualIgnitionSuccessRate >= minIgnitionSuccessRate"]
    }
    requirement "CLR-R057" {
        description: "The Apollo 11 Mission's lunar lander's descent propulsion system shall provide throttleable thrust\n\t\t\tfrom 10% to 100% of maximum thrust to enable precise lunar landing control."
        title: "DPSThrottlingRange"
        minRequiredThrottlePercentage: 10 percent
        maxRequiredThrottlePercentage: 100 percent
        require: ["actualMinThrottlePercentage <= minRequiredThrottlePercentage and actualMaxThrottlePercentage >= maxRequiredThrottlePercentage"]
    }
    requirement "CLR-R058" {
        description: "The Apollo 11 Mission's lunar lander's ascent propulsion system shall achieve a burn duration of <= 400 seconds\n\t\t\tto insert the ascent stage into lunar orbit."
        title: "LMASBurnDuration"
        maxAscentBurnDuration: 400 s
        require: ["actualAscentBurnDuration <= maxAscentBurnDuration"]
    }
    requirement "CLR-R059" {
        description: "All primary structural elements of the Apollo 11 Mission launch system shall withstand launch loads of up to 4 gees\n\t\t\twithout permanent deformation or failure."
        title: "PrimaryStructureLoadLimit"
        requiredMaxLaunchLoadWithstood: "4 gn"
        require: ["actualMaxLaunchLoadWithstood >= requiredMaxLaunchLoadWithstood and not permanentDeformationObserved and not structuralFailureObserved"]
    }
    requirement "CLR-R060" {
        description: "The Apollo 11 Mission's crew vehicle heat shield shall ablate as designed and limit internal crew compartment\n\t\t\ttemperature rise to less than 5°C during atmospheric re-entry."
        title: "CMHeatShieldThermalPerf"
        maxInternalTemperatureRise: 5 °C
        require: ["actualInternalTemperatureRise < maxInternalTemperatureRise and heatShieldAblationAsDesigned"]
    }
    requirement "CLR-R061" {
        description: "All pyrotechnic devices (e.g., stage separation, parachute deployment) used in the Apollo 11 Mission\n\t\t\tshall have a 99.99% initiation reliability."
        title: "PyrotechnicInitiationReliability"
        minInitiationReliability: 99.99 percent
        require: ["actualInitiationReliability >= minInitiationReliability"]
    }
    requirement "CLR-R062" {
        description: "The Apollo 11 Mission's spacecraft docking system shall enable two spacecraft to mate and form a rigid connection\n\t\t\tin lunar orbit, capable of withstanding dynamic loads during rendezvous maneuvers,\n\t\t\twith a 99.5% success rate for capture and latching."
        title: "DockingMechanismReliability"
        minCaptureLatchingSuccessRate: 99.5 percent
        require: ["actualCaptureLatchingSuccessRate >= minCaptureLatchingSuccessRate and connectionRigidityMaintained and withstandsDynamicLoads"]
    }
    requirement "CLR-R063" {
        description: "The Apollo 11 Mission's lunar lander's landing gear shall absorb impact energy equivalent to a 3 meters per second vertical velocity\n\t\t\tand 1 meter per second horizontal velocity at touchdown without structural failure."
        title: "LMDSLandingGearEnergy"
        minImpactVerticalVelocityRequired: 3 m/s
        minImpactHorizontalVelocityRequired: 1 m/s
        require: ["maxImpactVerticalVelocityAbsorbed >= minImpactVerticalVelocityRequired and maxImpactHorizontalVelocityAbsorbed >= minImpactHorizontalVelocityRequired and not structuralFailureOccurred"]
    }
    requirement "CLR-R064" {
        description: "The Apollo 11 Mission's crew's potable water system shall provide a minimum of 2 kilograms of potable water per crew member per day."
        title: "ECSWaterSupplyRate"
        minWaterRequiredPerCrewPerDay: 2 kg
        require: ["actualWaterProvidedPerCrewPerDay >= minWaterRequiredPerCrewPerDay"]
    }
    requirement "CLR-R065" {
        description: "The Apollo 11 Mission's cabin atmosphere purification system shall maintain carbon dioxide levels below 0.5% by volume\n\t\t\tand remove particulate matter >= 0.5 microns with 99% efficiency."
        title: "ECSFilterEfficiency"
        maxCO2LevelReqd: 0.5 percent
        minParticulateRemovalEfficiency: 99 percent
        require: ["actualCO2LevelMax <= maxCO2LevelReqd and actualParticulateRemovalEfficiency >= minParticulateRemovalEfficiency"]
    }
    requirement "CLR-R066" {
        description: "The Apollo 11 Mission's data acquisition system shall record and store all critical telemetry data for the entire mission duration,\n\t\t\twith no less than 99.9% data integrity."
        title: "DataHandlingIntegrity"
        minDataIntegrityPercentage: 99.9 percent
        require: ["actualDataIntegrityPercentage >= minDataIntegrityPercentage"]
    }
    requirement "CLR-R067" {
        description: "The Apollo 11 Mission spacecraft shall be capable of transmitting live video from the lunar surface to Earth\n\t\t\twith a resolution of at least 320x240 pixels and a frame rate of >= 10 frames per second."
        title: "CommVideoBandwidth"
        minVideoWidth: 320
        minVideoHeight: 240
        minFrameRate: "10 fps"
        require: ["actualVideoWidth >= minVideoWidth and actualVideoHeight >= minVideoHeight and actualFrameRate >= minFrameRate"]
    }
    requirement "CLR-R068" {
        description: "The Apollo 11 Mission's Extravehicular Mobility Unit shall maintain internal pressure at 25.5 kilopascals with no more than 5% pressure loss per hour during Extravehicular Activity."
        title: "EMUPressureContainment"
        nominalEMUInternalPressure: 25.5 kPa
        maxPressureLossRate: 5 percent
        require: ["actualPressureLossRate <= maxPressureLossRate"]
    }
    requirement "CLR-R069" {
        description: "The Apollo 11 Mission's deployed scientific instruments shall be operational for a minimum of 30 days post-deployment\n\t\t\tand transmit at least 100 Kilobytes of scientific data per day for this period."
        title: "EASEPOperationalDuration"
        minOperationalDuration: "30 d"
        minDailyDataTransmission: 100 kB
        require: ["actualOperationalDuration >= minOperationalDuration and actualDailyDataTransmission >= minDailyDataTransmission"]
    }
    requirement "CLR-R070" {
        description: "All mission-critical flight systems of the Apollo 11 Mission (e.g., primary propulsion, life support, main computer)\n\t\t\tshall demonstrate a statistical reliability of >= 0.999 (three nines) for their operational lifetimes."
        title: "MissionSystemReliability"
        minRequiredReliability: 0.999
        require: ["actualSystemReliability >= minRequiredReliability"]
    }
    requirement "CLR-R071" {
        description: "The Apollo 11 Mission's flight software for primary guidance and control shall have zero critical defects (Severity 1 or 2)\n\t\t\tidentified during a minimum of 2,000 hours of integrated hardware-in-the-loop simulation prior to flight."
        title: "FlightSoftwareDefectRate"
        maxCriticalDefectsAllowed: 0
        minSimulationHoursRequired: 2000 h
        require: ["actualCriticalDefectsCount <= maxCriticalDefectsAllowed and actualSimulationHours >= minSimulationHoursRequired"]
    }
    requirement "CLR-R072" {
        description: "All flight components of the Apollo 11 Mission shall undergo acceptance testing to verify conformance to specifications\n\t\t\tbefore integration, with <= 0.1% non-conformance rate."
        title: "ComponentAcceptanceRate"
        maxNonConformanceRate: 0.1 percent
        require: ["actualNonConformanceRate <= maxNonConformanceRate"]
    }
    requirement "CLR-R073" {
        description: "All spacecraft materials of the Apollo 11 Mission shall withstand long-term exposure to the space environment\n\t\t\t(vacuum, radiation, thermal cycling) for up to 14 days without critical degradation of structural integrity\n\t\t\tor functional performance."
        title: "MaterialVacuumResistance"
        maxDegradationLevel: 0 percent
        minExposureDurationReqd: "14 d"
        require: ["actualDegradationLevel <= maxDegradationLevel and actualExposureDuration >= minExposureDurationReqd"]
    }
    requirement "CLR-R074" {
        description: "The entire Apollo 11 Mission system shall be designed to withstand the electromagnetic interference (EMI) environment\n\t\t\tof launch and space operations without functional degradation."
        title: "ElectronicEMIImmunity"
        require: ["not functionalDegradationFromEMI"]
    }
    requirement "CLR-R075" {
        description: "The visual data capture system shall provide a minimum resolution of 2k for still images \n\t\t\tand 16mm equivalent for motion film."
        title: "ImageResolutionStandard"
    }
    requirement "CLR-R076" {
        description: "The waste containment system shall provide a minimum of 0.5 cubic feet of sealed storage \n\t\t\tper crew member for an 8-day mission."
        title: "WasteStorageVolume"
    }
    requirement "CLR-R077" {
        description: "The cross-vehicle audio link shall operate within the 250-300 MHz range with a signal-to-noise \n\t\t\tratio of at least 20dB."
        title: "VHFAudioBandwidth"
    }
    requirement "CLR-R078" {
        description: "All Level-1 critical electronic components shall be duplicated in an 'A' and 'B' bus \n\t\t\tconfiguration with automatic failover."
        title: "SystemRedundancyMapping"
    }
    requirement "CLR-R079" {
        description: "The onboard medical kit shall contain antibiotics, analgesics, and minor surgical tools \n\t\t\tcertified for use in zero-gravity."
        title: "MedicalKitSpecification"
    }
    requirement "CLR-R080" {
        description: "Mission telemetry shall be recorded on magnetic tape at three geographically distinct \n\t\t\tsites with 99.9% data parity."
        title: "GroundArchivalProtocol"
    }
    requirement "CLR-R081" {
        description: "The crew seating system shall attenuate peak launch acceleration to ensure loads \n                   experienced by the crew do not exceed 6g."
        title: "CrewSeatLoadFactor"
    }
    requirement "CLR-R082" {
        description: "The Instrument Unit shall trigger an automatic abort if the vehicle attitude deviation \n\t\t\t\texceeds 5 degrees or if tank pressure drops below 90% of nominal."
        title: "LaunchAbortThresholds"
    }
    requirement "CLR-R083" {
        description: "Pyrotechnic separation events shall be triggered within 50 milliseconds of the \n\t\t\t\tcommand issue to prevent re-contact."
        title: "SeparationTimerPrecision"
    }
    requirement "CLR-R084" {
        description: "The crew cabin shall maintain a pressure leak rate of less than 0.1 psi per hour at a nominal pressure of 5.0 psi."
        title: "CabinPressureHold"
    }
    requirement "CLR-R085" {
        description: "The master mission clock system shall maintain synchronization across all segments with a drift of less than 1ms per 24 hours."
        title: "TimingSynchError"
    }
    requirement "CLR-R086" {
        description: "The S-IC engine ignition sequence shall complete within 150ms of the authenticated launch command."
        title: "EngineStartLatency"
    }
    requirement "CLR-R087" {
        description: "Separation pyrotechnics shall not exceed a peak shock load of 50g on adjacent structural interfaces."
        title: "StagingShockLimit"
    }
    requirement "CLR-R088" {
        description: "Residual thrust from the S-IVB engine post-cutoff shall not exceed 0.05 m/s."
        title: "CutoffDeltaVError"
    }
    requirement "CLR-R089" {
        description: "Mission-critical flight displays in the CM cockpit shall have a minimum data refresh rate of 2 Hz."
        title: "CockpitDisplayRefresh"
    }
    requirement "CLR-R090" {
        description: "High-rate telemetry channels for engine health shall be sampled at a minimum of 100 samples per second."
        title: "TelemetrySamplingRate"
    }
    requirement "CLR-R091" {
        description: "The S-IVB auxiliary propulsion system shall provide a minimum delta-V of 1.0 m/s for post-TLI separation."
        title: "PostTliSeparationVelocity"
    }
    requirement "CLR-R092" {
        description: "The SLA separation springs shall provide a minimum separation velocity of 0.3 meters per second."
        title: "SlaSeparationVelocity"
    }
    requirement "CLR-R093" {
        description: "The RCS shall provide a minimum angular acceleration of 2.0 deg/sec² for transposition maneuvers."
        title: "TranspositionManeuverRate"
    }
    requirement "CLR-R094" {
        description: "The onboard sextant shall provide an angular measurement accuracy of +/- 10 arcseconds."
        title: "SextantAngularAccuracy"
    }
    requirement "CLR-R095" {
        description: "The navigation computer shall integrate new tracking data into the state vector within 500ms of receipt."
        title: "StateVectorIntegrationLatency"
    }
    requirement "CLR-R096" {
        description: "The MSFN ground stations shall maintain an uplink command turnaround latency of less than 2.0 seconds."
        title: "CommandTurnaroundLatency"
    }
    requirement "CLR-R097" {
        description: "The Unified S-Band system shall have a minimum receiver sensitivity of -120 dBm."
        title: "GroundStationSensitivity"
    }
    requirement "CLR-R098" {
        description: "The ground communication network shall maintain a Bit Error Rate (BER) of less than 10⁻⁶ across all relay sites."
        title: "RelayNetworkBitErrorRate"
    }
    requirement "CLR-R099" {
        description: "The mission timeline management software shall trigger an alert if execution drifts more than 5 minutes from plan."
        title: "TimelineDriftAlert"
    }
    requirement "CLR-R100" {
        description: "Ambient noise levels in the crew sleep area shall be limited to 55 dBA."
        title: "SleepAcousticLimit"
    }
    requirement "CLR-R101" {
        description: "The LOI burn shall target a lunar orbit perigee of 60 +/- 5 nautical miles."
        title: "TargetOrbitPerigee"
    }
    requirement "CLR-R102" {
        description: "The LM crew transfer hatch shall provide a minimum clear opening of 32 x 32 inches."
        title: "HatchClearanceDimensions"
    }
    requirement "CLR-R103" {
        description: "The LM primary guidance computer shall be initialized and aligned within 60 minutes of activation."
        title: "LmComputerAlignmentTime"
    }
    requirement "CLR-R104" {
        description: "The LM docking latches shall require a minimum of 300 inch-pounds of torque to achieve a rigid seal."
        title: "DockingLatchTorqueThreshold"
    }
    requirement "CLR-R105" {
        description: "The undocking maneuver shall target a relative separation velocity of 0.1 +/- 0.05 meters per second."
        title: "UndockingSeparationVelocity"
    }
    requirement "CLR-R106" {
        description: "The LM DSKY displays shall be readable under direct sunlight (minimum 500 nits brightness)."
        title: "DskyDisplayBrightness"
    }
    requirement "CLR-R107" {
        description: "The landing radar shall achieve frequency lock within 5 seconds of the Radar Enable command."
        title: "LandingRadarLockTime"
    }
    requirement "CLR-R108" {
        description: "The LM post-landing safe sequence shall complete within 60 seconds of touchdown."
        title: "PostLandingSafeTime"
    }
    requirement "CLR-R109" {
        description: "The LM cabin integrity test shall confirm a leak rate of <= 0.05 psi/hour before EVA egress."
        title: "LmCabinIntegrityLeakRate"
    }
    requirement "CLR-R110" {
        description: "The flag assembly shall withstand a 10-Newton lateral deployment force without collapsing."
        title: "SymbolicMountRigidity"
    }
    requirement "CLR-R111" {
        description: "Lunar sample tongs shall provide a minimum 5:1 mechanical leverage for rock collection."
        title: "SampleToolMechanicalAdvantage"
    }
    requirement "CLR-R112" {
        description: "EASEP experiment packages shall utilize 20-layer aluminized Mylar for thermal insulation."
        title: "EasepThermalInsulation"
    }
    requirement "CLR-R113" {
        description: "Astronaut radio systems shall maintain voice link at a maximum range of 500 meters from the LM."
        title: "EvaCommunicationRange"
    }
    requirement "CLR-R114" {
        description: "Surface operations shall synchronize to UTC time with an accuracy of +/- 0.5 seconds."
        title: "SurfaceTimelineSync"
    }
    requirement "CLR-R115" {
        description: "The LM ascent stage shall provide a minimum delta-V of 1,850 m/s to achieve lunar orbit."
        title: "LmAscentDeltaVMinimum"
    }
    requirement "CLR-R116" {
        description: "The rendezvous radar shall target an orbital node within 5 km of the CSM."
        title: "RendezvousOrbitTarget"
    }
    requirement "CLR-R117" {
        description: "The CM reentry flight path angle shall be targeted at -6.5 degrees +/- 0.2 degrees."
        title: "ReentryFlightPathAngle"
    }
}

logical_architecture "Logical Architecture" {
    function "LoadConsumablesAndPropellants" {
        id: "LoadConsumablesAndPropellants"
        description: "The process of fueling the Saturn V rocket and loading all necessary consumables like oxygen and water into the spacecraft."
    }
    function "TransferCrewToVehicle" {
        id: "TransferCrewToVehicle"
        description: "The operation where the flight crew moves from the crew quarters to the launch pad and ingresses the Command Module."
    }
    function "PerformPreLaunchCountdown" {
        id: "PerformPreLaunchCountdown"
        description: "The synchronized sequence of checks and configurations performed in the final hours and minutes leading up to liftoff."
    }
    function "ExecuteLaunchSequence" {
        id: "ExecuteLaunchSequence"
        description: "The automated and monitored sequence of engine ignitions, vehicle ascent, and staging events that propel the vehicle into orbit."
    }
    function "MonitorAscentTrajectory" {
        id: "MonitorAscentTrajectory"
        description: "The continuous tracking of the vehicle's speed, altitude, and flight path during ascent to ensure it is following the correct trajectory."
    }
    function "ExecuteTLIBurn" {
        id: "ExecuteTLIBurn"
        description: "The critical engine burn of the S-IVB third stage to propel the spacecraft out of Earth orbit and onto a trajectory to the Moon."
    }
    function "PerformTranspositionDockingExtraction" {
        id: "PerformTranspositionDockingExtraction"
        description: "The complex maneuver where the CSM separates, turns around, docks with the LM, and extracts it from the S-IVB stage."
    }
    function "ConductMidCourseCorrections" {
        id: "ConductMidCourseCorrections"
        description: "Small propulsive burns performed during the coast phases to refine the spacecraft's trajectory."
    }
    function "MonitorSpacecraftSystems" {
        id: "MonitorSpacecraftSystems"
        description: "The continuous monitoring of all onboard systems by the crew and Mission Control to ensure vehicle health."
    }
    function "CommunicateWithMissionControl" {
        id: "CommunicateWithMissionControl"
        description: "The two-way exchange of voice, telemetry, and command data between the spacecraft and ground stations."
    }
    function "ManageCrewRestPeriods" {
        id: "ManageCrewRestPeriods"
        description: "The scheduling and management of sleep, meals, and downtime for the crew to ensure their health and performance."
    }
    function "ExecuteLOIBurn" {
        id: "ExecuteLOIBurn"
        description: "The engine burn of the Service Module to slow the spacecraft, allowing it to be captured into a stable lunar orbit."
    }
    function "ConductLunarOrbitCheckouts" {
        id: "ConductLunarOrbitCheckouts"
        description: "A series of system health checks performed on the CSM and LM while in lunar orbit."
    }
    function "PrepareLMforUndocking" {
        id: "PrepareLMforUndocking"
        description: "The process of activating the Lunar Module's systems and configuring it for independent flight."
    }
    function "PerformLMUndocking" {
        id: "PerformLMUndocking"
        description: "The physical separation of the Lunar Module from the Command Module in lunar orbit."
    }
    function "ExecutePoweredDescentBurn" {
        id: "ExecutePoweredDescentBurn"
        description: "The continuous, throttleable engine burn of the LM Descent Stage to control the descent from orbit to the lunar surface."
    }
    function "MonitorDescentTelemetry" {
        id: "MonitorDescentTelemetry"
        description: "The real-time tracking of the LM's altitude and velocity during the landing phase."
    }
    function "PerformManualPiloting" {
        id: "PerformManualPiloting"
        description: "The manual control of the LM by the Commander during the final phase of landing to avoid hazards."
    }
    function "ConductLMPostLandingChecks" {
        id: "ConductLMPostLandingChecks"
        description: "A series of procedures performed after landing to safe the LM's systems and verify its integrity."
    }
    function "PrepareForEVA" {
        id: "PrepareForEVA"
        description: "The process of the crew donning their spacesuits and depressurizing the LM cabin for surface exploration."
    }
    function "PerformLunarEVA" {
        id: "PerformLunarEVA"
        description: "The extravehicular activity where the crew operates on the lunar surface to explore and conduct experiments."
    }
    function "CollectLunarSamples" {
        id: "CollectLunarSamples"
        description: "The task of gathering and storing rock and soil samples from the lunar surface."
    }
    function "DeployScientificInstruments" {
        id: "DeployScientificInstruments"
        description: "The setup of the Early Apollo Scientific Experiments Package (EASEP) on the lunar surface."
    }
    function "ConcludeEVA" {
        id: "ConcludeEVA"
        description: "The process of the crew re-entering the LM, stowing equipment, and closing the hatch."
    }
    function "ManageLunarSurfaceRestPeriod" {
        id: "ManageLunarSurfaceRestPeriod"
        description: "The scheduled rest and sleep period for the crew while on the Moon."
    }
    function "ExecuteLunarAscentBurn" {
        id: "ExecuteLunarAscentBurn"
        description: "The firing of the LM Ascent Stage engine to lift off from the lunar surface and return to orbit."
    }
    function "PerformRendezvousManeuvers" {
        id: "PerformRendezvousManeuvers"
        description: "A series of precise orbital maneuvers performed by the LM to match the orbit of and approach the CSM."
    }
    function "ConductLMCSMDocking" {
        id: "ConductLMCSMDocking"
        description: "The physical connection of the LM Ascent Stage with the orbiting CSM."
    }
    function "TransferCrewAndSamples" {
        id: "TransferCrewAndSamples"
        description: "The movement of the crew and collected lunar samples from the LM back into the Command Module."
    }
    function "JettisonLM" {
        id: "JettisonLM"
        description: "The final separation from the LM Ascent Stage, leaving it in lunar orbit."
    }
    function "ExecuteTEIBurn" {
        id: "ExecuteTEIBurn"
        description: "The engine burn of the Service Module to propel the CSM out of lunar orbit and onto a trajectory back to Earth."
    }
    function "PrepareCMforReentry" {
        id: "PrepareCMforReentry"
        description: "Final checks and configurations of the Command Module in preparation for atmospheric entry."
    }
    function "SeparateModules" {
        id: "SeparateModules"
        description: "The pyrotechnic separation of the Command Module from the Service Module just before reentry."
    }
    function "GuideReentryTrajectory" {
        id: "GuideReentryTrajectory"
        description: "The controlled, lifting flight of the Command Module through the atmosphere to manage heating and target the landing zone."
    }
    function "DeployParachutes" {
        id: "DeployParachutes"
        description: "The sequential deployment of drogue and main parachutes to slow the Command Module for a safe splashdown."
    }
    function "ConductSplashdownOperations" {
        id: "ConductSplashdownOperations"
        description: "The final water landing of the Command Module and the activation of its recovery systems."
    }
    function "RetrieveCrewAndCM" {
        id: "RetrieveCrewAndCM"
        description: "The location and recovery of the crew and capsule from the ocean by naval forces."
    }
    function "InitiateQuarantineProcedures" {
        id: "InitiateQuarantineProcedures"
        description: "The implementation of biological isolation protocols for the crew, capsule, and samples upon their return."
    }
    function "SecureLunarSamples" {
        id: "SecureLunarSamples"
        description: "The process of packaging and transporting the returned lunar samples to the Lunar Receiving Laboratory."
    }
    component "apollo11MissionIndividual" {
        id: "apollo11MissionIndividual"
        description: "A timeline representing the key events of the actual Apollo 11 mission from July 16, 1969, to July 24, 1969."
        timeslice "crewIngress" {
            description: "The flight crew enters the Command Module 'Columbia'."
            snapshot "atIngress" {
                description: "The flight crew begins entering the Command Module 'Columbia'."
                missionTime: -9000 s
            }
            snapshot "atIngressComplete" {
                description: "Crew is secured and the Command Module hatch is sealed."
                missionTime: -3600 s
            }
        }
        timeslice "liftoff" {
            description: "This timeslice covers the ascent through Earth's atmosphere."
            snapshot "atT0" {
                description: "State Activation: LaunchPhase begins with liftoff."
                missionTime: 0 s
            }
            snapshot "atSICSeparation" {
                description: "Separation Event: The S-IC first stage separates."
                missionTime: 168 s
            }
            snapshot "atSIISeparation" {
                description: "Separation Event: The S-II second stage separates."
                missionTime: 540 s
            }
        }
        timeslice "translunarInjection" {
            description: "This timeslice covers the burn to leave Earth orbit."
            snapshot "atTLI" {
                description: "State Activation: TransLunarInjectionPhase begins with the S-IVB burn."
                missionTime: 10036 s
            }
            snapshot "atTLIComplete" {
                description: "The TLI burn is complete, placing the vehicle on a translunar trajectory."
                missionTime: 10384 s
            }
        }
        timeslice "transpositionAndDocking" {
            description: "This timeslice covers the CSM/LM docking and extraction maneuver."
            snapshot "atCSMSeparation" {
                description: "Separation Event: The CSM separates from the S-IVB/SLA."
                missionTime: 11400 s
            }
            snapshot "atCSLMDocking" {
                description: "The CSM has turned around and is docking with the LM."
                missionTime: 12000 s
            }
            snapshot "atExtractionComplete" {
                description: "The docked CSM and LM have been extracted from the S-IVB stage."
                missionTime: 14400 s
            }
        }
        timeslice "translunarCoast" {
            description: "This timeslice represents the coast to the Moon."
            snapshot "atTLCStart" {
                description: "State Activation: TransLunarCoastPhase begins."
                missionTime: 14401 s
            }
        }
        timeslice "lunarOrbitInsertion" {
            description: "This timeslice covers the maneuver to enter lunar orbit."
            snapshot "atLOI" {
                description: "State Activation: LunarOrbitInsertionPhase begins with the LOI burn."
                missionTime: 273110 s
            }
            snapshot "atLOIComplete" {
                description: "The LOI burn is complete and the spacecraft is in a stable orbit."
                missionTime: 273470 s
            }
        }
        timeslice "descentPreparation" {
            description: "This timeslice covers checking out and undocking the LM."
            snapshot "atDescentPrep" {
                description: "State Activation: LunarOrbitOperationsDescentPreparationPhase begins."
                missionTime: 352800 s
            }
            snapshot "atLMUndocking" {
                description: "Separation Event: The LM undocks from the CSM."
                missionTime: 353100 s
            }
        }
        timeslice "poweredDescent" {
            description: "This timeslice covers the powered descent to the lunar surface."
            softLandingAchieved: true
            snapshot "atPDI" {
                description: "State Activation: PoweredDescentPhase begins with Powered Descent Initiation."
                missionTime: 360205 s
            }
            snapshot "atLanding" {
                description: "The Lunar Module 'Eagle' touches down in the Sea of Tranquility."
                missionTime: 360940 s
            }
        }
        timeslice "lunarSurfaceOps" {
            description: "This timeslice covers the activities on the lunar surface."
            instrumentsDeployed: true
            snapshot "atEVAStart" {
                description: "State Activation: LunarSurfaceOperationsPhase begins with the first EVA."
                missionTime: 384300 s
            }
            snapshot "atEVAEnd" {
                description: "The EVA is complete and the crew are back inside the LM."
                missionTime: 393300 s
            }
        }
        timeslice "ascentAndRendezvous" {
            description: "This timeslice covers returning to lunar orbit and docking with the CSM."
            snapshot "atLunarLiftoff" {
                description: "State Activation: LunarAscentRendezvousPhase begins with liftoff from the Moon."
                missionTime: 447721 s
            }
            snapshot "atLunarDocking" {
                description: "The LM Ascent Stage has completed rendezvous and docks with the CSM."
                missionTime: 460200 s
            }
        }
        timeslice "transearthInjection" {
            description: "This timeslice covers the burn to leave lunar orbit."
            snapshot "atTEI" {
                description: "State Activation: TransEarthInjectionPhase begins with the TEI burn."
                missionTime: 478500 s
            }
        }
        timeslice "transearthCoast" {
            description: "This timeslice represents the coast back to Earth."
            snapshot "atTECStart" {
                description: "State Activation: TransEarthCoastPhase begins."
                missionTime: 478651 s
            }
        }
        timeslice "reentryAndLanding" {
            description: "This timeslice covers the atmospheric entry and splashdown."
            snapshot "atCMSMSeparation" {
                description: "Separation Event: The Command Module separates from the Service Module."
                missionTime: 688200 s
            }
            snapshot "atReentry" {
                description: "State Activation: ReentryLandingPhase begins as the CM hits the atmosphere."
                missionTime: 689100 s
            }
            snapshot "atSplashdown" {
                description: "The Command Module 'Columbia' splashes down in the Pacific Ocean."
                missionTime: 690395 s
            }
        }
        timeslice "recoveryAndQuarantine" {
            description: "This timeslice covers the recovery of the crew."
            crewReturnedSafely: true
            requiredSamplesReturned: true
            snapshot "atRecovery" {
                description: "State Activation: RecoveryQuarantinePhase begins with crew retrieval."
                missionTime: 693000 s
            }
            snapshot "atRecoveryComplete" {
                description: "The Command Module is hoisted aboard the recovery ship."
                missionTime: 694800 s
            }
        }
    }
    component "NASA" {
        id: "NASA"
        is: "GovernmentAgency"
    }
    component "Apollo11Crew" {
        id: "Apollo11Crew"
        is: "Stakeholder"
    }
    component "MissionControlPersonnel" {
        id: "MissionControlPersonnel"
    }
    component "AerospaceCompany" {
        id: "AerospaceCompany"
    }
    component "USGovernment" {
        id: "USGovernment"
        is: "GovernmentAgency"
    }
    component "ForeignNation" {
        id: "ForeignNation"
    }
    component "ScientificCommunity" {
        id: "ScientificCommunity"
    }
    component "AmericanPublic" {
        id: "AmericanPublic"
        is: "Public"
    }
    component "GlobalPublic" {
        id: "GlobalPublic"
        is: "Public"
    }
    component "Media" {
        id: "Media"
    }
    component "AstronautFamilies" {
        id: "AstronautFamilies"
        is: "Stakeholder"
    }
    component "FutureGenerations" {
        id: "FutureGenerations"
        is: "Stakeholder"
    }
    component "stages" {
        id: "stages"
        is: "RocketStage"
        component "engines" {
            id: "engines"
            is: "RocketEngine"
        }
    }
    component "LaunchSystem" {
        id: "LaunchSystem"
        description: "This system's sole purpose is to propel the payload from Earth's surface onto a precise path to the Moon. It is responsible for all functions related to ascent and injection into a translunar trajectory."
        function "provideStage1Thrust" {
            id: "provideStage1Thrust"
            is: "ProvideStage1Thrust"
            port in "launchReadyStatus"
            port out "stage1BurnoutStatus"
        }
        function "provideStage2Thrust" {
            id: "provideStage2Thrust"
            is: "ProvideStage2Thrust"
            port in "stage1BurnoutStatus"
            port out "stage2BurnoutStatus"
        }
        function "provideOrbitInsertionThrust" {
            id: "provideOrbitInsertionThrust"
            is: "ProvideOrbitInsertionThrust"
            port in "stage2BurnoutStatus"
            port out "orbitalStatus"
        }
        function "guideAscentTrajectory" {
            id: "guideAscentTrajectory"
            is: "GuideAscentTrajectory"
            port in "ascentStatus"
            port out "guidedStatus"
        }
        function "provideTranslunarInjectionThrust" {
            id: "provideTranslunarInjectionThrust"
            is: "ProvideTranslunarInjectionThrust"
            port in "currentStatus"
            port out "postBurnStatus"
        }
        function "controlTranslunarInjectionBurn" {
            id: "controlTranslunarInjectionBurn"
            is: "ControlTranslunarInjectionBurn"
            port in "currentStatus"
        }
    }
    component "Spacecraft" {
        id: "Spacecraft"
        description: "This system comprises the vehicles that travel to, operate at, and return from the Moon. It handles all functions related to in-space travel, life support, landing, and returning through Earth's atmosphere."
        function "provideAttitudeControlThrust" {
            id: "provideAttitudeControlThrust"
            is: "ProvideAttitudeControlThrust"
            port in "currentStatus"
            port out "newAttitudeStatus"
        }
        function "mechanicallyLatchModules" {
            id: "mechanicallyLatchModules"
            is: "MechanicallyLatchModules"
            port in "currentStatus"
            port out "dockedStatus"
        }
        function "calculateCourseCorrection" {
            id: "calculateCourseCorrection"
            is: "CalculateCourseCorrection"
            port in "currentStatus"
            port out "burnPlan"
        }
        function "executeCourseCorrectionBurn" {
            id: "executeCourseCorrectionBurn"
            is: "ExecuteCourseCorrectionBurn"
            port in "burnPlan"
            port out "correctedStatus"
        }
        function "transmitAndReceiveSpacecraftSignals" {
            id: "transmitAndReceiveSpacecraftSignals"
            is: "TransmitAndReceiveSpacecraftSignals"
            port in "currentStatus"
        }
        function "provideLunarOrbitInsertionThrust" {
            id: "provideLunarOrbitInsertionThrust"
            is: "ProvideLunarOrbitInsertionThrust"
            port in "currentStatus"
            port out "postBurnStatus"
        }
        function "controlLunarOrbitInsertionBurn" {
            id: "controlLunarOrbitInsertionBurn"
            is: "ControlLunarOrbitInsertionBurn"
            port in "currentStatus"
            port out "finalStatus"
        }
        function "separateLanderFromOrbiter" {
            id: "separateLanderFromOrbiter"
            is: "SeparateLanderFromOrbiter"
            port in "currentStatus"
            port out "lander"
            port out "orbiter"
        }
        function "executeDescentBurn" {
            id: "executeDescentBurn"
            is: "ExecuteDescentBurn"
            port in "currentStatus"
            port out "descentStatus"
        }
        function "executeAscentBurn" {
            id: "executeAscentBurn"
            is: "ExecuteAscentBurn"
            port in "currentStatus"
            port out "ascentVehicle"
        }
        function "calculateRendezvousManeuvers" {
            id: "calculateRendezvousManeuvers"
            is: "CalculateRendezvousManeuvers"
            port in "currentStatus"
            port out "rendezvousPlan"
        }
        function "executeRendezvousBurns" {
            id: "executeRendezvousBurns"
            is: "ExecuteRendezvousBurns"
            port in "rendezvousPlan"
            port out "proximalVehicle"
        }
        function "discardAscentStage" {
            id: "discardAscentStage"
            is: "DiscardAscentStage"
            port in "currentStatus"
            port out "finalVehicle"
        }
        function "executeTransearthBurn" {
            id: "executeTransearthBurn"
            is: "ExecuteTransearthBurn"
            port in "currentStatus"
            port out "finalStatus"
        }
        function "separateReturnCapsule" {
            id: "separateReturnCapsule"
            is: "SeparateReturnCapsule"
            port in "currentStatus"
            port out "capsule"
        }
        function "guideAtmosphericEntry" {
            id: "guideAtmosphericEntry"
            is: "GuideAtmosphericEntry"
            port in "currentStatus"
            port out "reentryStatus"
        }
        function "deployDecelerators" {
            id: "deployDecelerators"
            is: "DeployDecelerators"
            port in "currentStatus"
            port out "descentStatus"
        }
        function "performSplashdown" {
            id: "performSplashdown"
            is: "PerformSplashdown"
            port in "currentStatus"
            port out "finalStatus"
        }
    }
    component "GroundSupportSystem" {
        id: "GroundSupportSystem"
        description: "This system is the Earth-based infrastructure and personnel that manage, control, and support the mission from start to finish."
        function "performPropellantLoading" {
            id: "performPropellantLoading"
            is: "PerformPropellantLoading"
            port in "currentStatus"
            port out "loadedStatus"
        }
        function "performCrewIngress" {
            id: "performCrewIngress"
            is: "PerformCrewIngress"
            port out "isCrewAboard"
        }
        function "retrieveVehicleAndCrew" {
            id: "retrieveVehicleAndCrew"
            is: "RetrieveVehicleAndCrew"
            port in "currentStatus"
            port out "recoveredCrew"
            port out "recoveredVehicle"
        }
        function "executeQuarantine" {
            id: "executeQuarantine"
            is: "ExecuteQuarantine"
            port in "crew"
        }
        function "secureReturnedSamples" {
            id: "secureReturnedSamples"
            is: "SecureReturnedSamples"
            port in "samples"
            port out "finalSamples"
        }
        function "executeTerminalCountdown" {
            id: "executeTerminalCountdown"
            is: "ExecuteTerminalCountdown"
            port out "goForLaunch"
        }
        function "analyzeLaunchTelemetryFromGround" {
            id: "analyzeLaunchTelemetryFromGround"
            is: "AnalyzeLaunchTelemetryFromGround"
            port in "ascentStatus"
        }
        function "monitorDownlinkedTelemetry" {
            id: "monitorDownlinkedTelemetry"
            is: "MonitorDownlinkedTelemetry"
            port in "currentStatus"
        }
        function "monitorDescent" {
            id: "monitorDescent"
            is: "MonitorDescent"
            port in "currentStatus"
            port out "monitoredStatus"
        }
    }
    component "DataTransmissionNetwork" {
        id: "DataTransmissionNetwork"
        description: "The system to relay and transmit data between the SoS elements."
        function "trackAndRelayGroundSignals" {
            id: "trackAndRelayGroundSignals"
            is: "TrackAndRelayGroundSignals"
        }
    }
    component "EVASystem" {
        id: "EVASystem"
        description: "This system consists of the personal equipment required for an astronaut to live and work in the vacuum of space on the lunar surface."
        function "provideIndividualLifesupport" {
            id: "provideIndividualLifesupport"
            is: "ProvideIndividualLifesupport"
        }
    }
    component "Crew" {
        id: "Crew"
        description: "This system represents the human operators who pilot the spacecraft, conduct experiments, and make real-time decisions."
        function "observeAscentDataFromCockpit" {
            id: "observeAscentDataFromCockpit"
            is: "ObserveAscentDataFromCockpit"
            port in "ascentStatus"
        }
        function "executeTranspositionManeuver" {
            id: "executeTranspositionManeuver"
            is: "ExecuteTranspositionManeuver"
            port in "currentStatus"
            port out "maneuveringStatus"
        }
        function "monitorOnboardSystems" {
            id: "monitorOnboardSystems"
            is: "MonitorOnboardSystems"
            port in "currentStatus"
        }
        function "manageCrewSchedules" {
            id: "manageCrewSchedules"
            is: "ManageCrewSchedules"
            port in "currentStatus"
            port out "restedStatus"
        }
        function "performOrbitalSystemCheckouts" {
            id: "performOrbitalSystemCheckouts"
            is: "PerformOrbitalSystemCheckouts"
            port in "currentStatus"
            port out "checkedVehicle"
        }
        function "prepareLanderForSeparation" {
            id: "prepareLanderForSeparation"
            is: "PrepareLanderForSeparation"
            port in "currentStatus"
            port out "preppedVehicle"
        }
        function "manuallyPilotLander" {
            id: "manuallyPilotLander"
            is: "ManuallyPilotLander"
            port in "currentStatus"
            port out "pilotedStatus"
        }
        function "performPostLandingCheck" {
            id: "performPostLandingCheck"
            is: "PerformPostLandingCheck"
            port in "currentStatus"
        }
        function "prepareForExtravehicularActivity" {
            id: "prepareForExtravehicularActivity"
            is: "PrepareForExtravehicularActivity"
            port in "currentStatus"
        }
        function "conductSurfaceExploration" {
            id: "conductSurfaceExploration"
            is: "ConductSurfaceExploration"
            port in "currentStatus"
        }
        function "gatherLunarSamples" {
            id: "gatherLunarSamples"
            is: "GatherLunarSamples"
            port out "samples"
        }
        function "deploySurfaceExperiments" {
            id: "deploySurfaceExperiments"
            is: "DeploySurfaceExperiments"
            port out "scienceData"
        }
        function "completeExtravehicularActivity" {
            id: "completeExtravehicularActivity"
            is: "CompleteExtravehicularActivity"
            port in "currentStatus"
        }
        function "conductSurfaceRestPeriod" {
            id: "conductSurfaceRestPeriod"
            is: "ConductSurfaceRestPeriod"
        }
        function "connectSpacecraft" {
            id: "connectSpacecraft"
            is: "ConnectSpacecraft"
            port in "currentStatus"
            port out "dockedVehicle"
        }
        function "transferCrewAndMaterials" {
            id: "transferCrewAndMaterials"
            is: "TransferCrewAndMaterials"
            port in "currentStatus"
            port in "samples"
            port out "postTransferVehicle"
        }
        function "prepareVehicleForReentry" {
            id: "prepareVehicleForReentry"
            is: "PrepareVehicleForReentry"
            port in "currentStatus"
            port out "finalStatus"
        }
    }
    component "ApolloProgram" {
        id: "ApolloProgram"
        description: "The Apollo program, also known as Project Apollo, was the United States human spaceflight program led by NASA, which landed the first humans on the Moon in 1969."
        component "Apollo11Mission" {
            id: "Apollo11Mission"
            description: "Apollo 11 was a spaceflight conducted from July 16 to 24, 1969, by the United States and launched by NASA."
            plannedDuration: 195.3 h
            researchAndDevelopmentCost: "5E9 $"
            manufacturingCost: "3E9 $"
            operationsCost: "1E9 $"
            personnelCost: "2E9 $"
            component "Apollo11MissionSystem" {
                id: "Apollo11MissionSystem"
                component "SA-506" {
                    id: "SA-506"
                    description: "Apollo 11. First crewed lunar landing, at Sea of Tranquility."
                }
                component "ApolloSpacecraft" {
                    id: "ApolloSpacecraft"
                    description: "The complete payload stack for the Apollo mission, including the CSM, LM, SLA, and Launch Escape System."
                    component "CSM-107" {
                        id: "CSM-107"
                        description: "Columbia\tApollo 11. Command module on display at National Air and Space Museum, Washington, D.C."
                    }
                    component "LM-5" {
                        id: "LM-5"
                        description: "Eagle, Apollo 11. Descent stage on lunar surface in Sea of Tranquility, ascent stage left in lunar orbit (could be still orbiting the moon)"
                    }
                    component "ApolloLaunchEscapeSystem" {
                        id: "ApolloLaunchEscapeSystem"
                        description: "A solid-fueled rocket tower mounted atop the Command Module to pull it away from the launch vehicle in an emergency."
                        mass: 4170 kg
                        powerLoad: 50 W
                        failureRate: "1E-7 1/h"
                        port in "cmInterfacePort"
                    }
                    component "ApolloCommandServiceModule" {
                        id: "ApolloCommandServiceModule"
                        description: "The combined Command and Service Module (CSM), which functions as the primary spacecraft for lunar transit and Earth return."
                        powerGenerated: "ApolloServiceModule.powerGenerated"
                        powerLoad: "ApolloCommandModule.powerLoad + ApolloServiceModule.powerLoad"
                        component "ApolloCommandModule" {
                            id: "ApolloCommandModule"
                            description: "The conical crew cabin, control center, and reentry vehicle."
                            mass: 5840 kg
                            powerLoad: 2000 W
                            failureRate: "1.5E-6 1/h"
                            port in "umbilicalPort"
                            port inout "dockingPort"
                            port out "lesInterfacePort"
                        }
                        component "ApolloServiceModule" {
                            id: "ApolloServiceModule"
                            description: "The primary support module for the Command Module, containing the main engine, power systems, and consumables."
                            dryMass: 6160 kg
                            propellantMass: 18440 kg
                            maxThrust: 91 kN
                            specificImpulse: 314 s
                            powerGenerated: 3000 W
                            powerLoad: 500 W
                            failureRate: "2.0E-6 1/h"
                            port out "umbilicalPort"
                            port in "launchVehicleInterfacePort"
                        }
                    }
                    component "ApolloLunarModule" {
                        id: "ApolloLunarModule"
                        description: "The two-stage vehicle that performed the lunar landing and ascent."
                        powerGenerated: "LunarModuleDescentStage.powerGenerated + LunarModuleAscentStage.powerGenerated"
                        powerLoad: "LunarModuleDescentStage.powerLoad + LunarModuleAscentStage.powerLoad"
                        component "LunarModuleDescentStage" {
                            id: "LunarModuleDescentStage"
                            description: "The lower portion of the Lunar Module, used for the powered landing and as a launch platform for the ascent stage."
                            dryMass: 2100 kg
                            propellantMass: 8250 kg
                            maxThrust: 45 kN
                            specificImpulse: 305 s
                            powerGenerated: 1500 W
                            powerLoad: 1000 W
                            failureRate: "3.0E-6 1/h"
                            port out "upperStagePort"
                        }
                        component "LunarModuleAscentStage" {
                            id: "LunarModuleAscentStage"
                            description: "The upper portion of the Lunar Module, which served as the crew cabin and ascent vehicle to leave the lunar surface."
                            dryMass: 2220 kg
                            propellantMass: 2370 kg
                            maxThrust: 16 kN
                            specificImpulse: 311 s
                            powerGenerated: 1500 W
                            powerLoad: 1200 W
                            failureRate: "2.5E-6 1/h"
                            port in "lowerStagePort"
                            port inout "dockingPort"
                        }
                    }
                    component "ApolloSpacecraftLMAdapter" {
                        id: "ApolloSpacecraftLMAdapter"
                        description: "A conical structure that housed the Lunar Module during launch and connected the Apollo spacecraft to the Saturn V."
                        mass: 1840 kg
                        port inout "launchVehicleInterfacePort"
                        port out "csmInterfacePort"
                    }
                }
                component "ExtravehicularMobilityUnit" {
                    id: "ExtravehicularMobilityUnit"
                    description: "The 'A7L' spacesuit, a self-contained personal spacecraft providing life support for lunar EVAs."
                    massOnEarth: 83 kg
                    nominalOperatingPressure: 25.5 kPa
                    lifeSupportDuration: 7 h
                    component "PressureSuiteAssembly" {
                        id: "PressureSuiteAssembly"
                        description: "The main suit portion, providing pressure and mobility."
                        port in "umbilicalPort"
                    }
                    component "PortableLifeSupportSystem" {
                        id: "PortableLifeSupportSystem"
                        description: "The 'backpack' containing oxygen, CO2 scrubbers, cooling water, and communications equipment."
                        port out "umbilicalPort"
                    }
                }
            }
            component "Apollo11MissionContext" {
                id: "Apollo11MissionContext"
                component "vehicleAssemblyBuilding" {
                    id: "vehicleAssemblyBuilding"
                    is: "VehicleAssemblyBuilding"
                }
                component "launchControlCenter" {
                    id: "launchControlCenter"
                    is: "LaunchControlCenter"
                }
                component "crawlerTransporter" {
                    id: "crawlerTransporter"
                    is: "CrawlerTransporter"
                }
                component "MissionControl" {
                    id: "MissionControl"
                    description: "The Earth-based command, control, and analysis center for post-launch operations."
                    location: "Houston, Texas"
                }
                component "RecoveryForces" {
                    id: "RecoveryForces"
                    description: "The assets tasked with retrieving the crew and capsule after splashdown."
                }
                component "MannedSpaceFlightNetwork" {
                    id: "MannedSpaceFlightNetwork"
                    description: "The Manned Space Flight Network (abbreviated MSFN, pronounced \"misfin\") was a set of tracking stations built to support the American Mercury, Gemini, Apollo, and Skylab space programs."
                }
            }
            component "Astronaut" {
                id: "Astronaut"
            }
            function "PerformLunarMission" {
                id: "PerformLunarMission"
                description: "The top-level function to transport a human crew to the lunar surface, enable exploration, and return them safely to Earth."
                port in "plan"
                port in "vehicleSystem"
                port in "flightCrew"
                port out "finalReport"
                function "ExecuteOutboundJourney" {
                    id: "ExecuteOutboundJourney"
                    description: "To execute all operations required to travel from a pre-launch state on Earth to achieving a stable orbit around the Moon."
                    port in "launchReadySystem"
                    port in "crew"
                    port out "vehicleInLunarOrbit"
                    function "PrepareForLaunch" {
                        id: "PrepareForLaunch"
                        description: "Covers final ground preparations. Corresponds to the PrepareForMissionPhase."
                        port in "assembledVehicle"
                        port in "crew"
                        port out "launchReadyVehicle"
                        port out "readyCrew"
                        function "load" {
                            id: "load"
                            is: "PerformPropellantLoading"
                            port in "currentStatus"
                            port out "loadedStatus"
                        }
                        function "board" {
                            id: "board"
                            is: "PerformCrewIngress"
                            port out "isCrewAboard"
                        }
                    }
                    function "LaunchToOrbit" {
                        id: "LaunchToOrbit"
                        description: "Applies motive force to overcome gravity and achieve a stable Earth orbit. Corresponds to the LaunchPhase."
                        port in "launchReadyVehicle"
                        port out "vehicleInEarthOrbit"
                        function "countdown" {
                            id: "countdown"
                            is: "ExecuteTerminalCountdown"
                            port out "goForLaunch"
                        }
                        function "provideThrust1" {
                            id: "provideThrust1"
                            is: "ProvideStage1Thrust"
                            port in "launchReadyStatus"
                            port out "stage1BurnoutStatus"
                        }
                        function "provideThrust2" {
                            id: "provideThrust2"
                            is: "ProvideStage2Thrust"
                            port in "stage1BurnoutStatus"
                            port out "stage2BurnoutStatus"
                        }
                        function "provideThrust3" {
                            id: "provideThrust3"
                            is: "ProvideOrbitInsertionThrust"
                            port in "stage2BurnoutStatus"
                            port out "orbitalStatus"
                        }
                        function "guide" {
                            id: "guide"
                            is: "GuideAscentTrajectory"
                            port in "ascentStatus"
                            port out "guidedStatus"
                        }
                        function "observe" {
                            id: "observe"
                            is: "ObserveAscentDataFromCockpit"
                            port in "ascentStatus"
                        }
                        function "analyze" {
                            id: "analyze"
                            is: "AnalyzeLaunchTelemetryFromGround"
                            port in "ascentStatus"
                        }
                    }
                    function "SetTranslunarCourse" {
                        id: "SetTranslunarCourse"
                        description: "Executes a propulsive maneuver to leave Earth orbit and reconfigure the vehicle. Corresponds to the TransLunarInjectionPhase."
                        port in "vehicleInEarthOrbit"
                        port out "vehicleOnTranslunarTrajectory"
                        function "executeTransposition" {
                            id: "executeTransposition"
                            is: "ExecuteTranspositionManeuver"
                            port in "currentStatus"
                            port out "maneuveringStatus"
                        }
                        function "provideAttitudeControlAndThrust" {
                            id: "provideAttitudeControlAndThrust"
                            is: "ProvideAttitudeControlThrust"
                            port in "currentStatus"
                            port out "newAttitudeStatus"
                        }
                        function "reconfigure" {
                            id: "reconfigure"
                            is: "MechanicallyLatchModules"
                            port in "currentStatus"
                            port out "dockedStatus"
                        }
                        function "burn" {
                            id: "burn"
                            is: "ProvideTranslunarInjectionThrust"
                            port in "currentStatus"
                            port out "postBurnStatus"
                        }
                        function "control" {
                            id: "control"
                            is: "ControlTranslunarInjectionBurn"
                            port in "currentStatus"
                        }
                    }
                    function "TraverseToMoon" {
                        id: "TraverseToMoon"
                        description: "Navigates the vehicle across cislunar space. Corresponds to the TransLunarCoastPhase."
                        port in "vehicleOnTranslunarTrajectory"
                        port out "vehicleInLunarSphere"
                        function "calculate" {
                            id: "calculate"
                            is: "CalculateCourseCorrection"
                            port in "currentStatus"
                            port out "burnPlan"
                        }
                        function "correct" {
                            id: "correct"
                            is: "ExecuteCourseCorrectionBurn"
                            port in "burnPlan"
                            port out "correctedStatus"
                        }
                        function "monitorOnboard" {
                            id: "monitorOnboard"
                            is: "MonitorOnboardSystems"
                            port in "currentStatus"
                        }
                        function "monitorGround" {
                            id: "monitorGround"
                            is: "MonitorDownlinkedTelemetry"
                            port in "currentStatus"
                        }
                        function "comms" {
                            id: "comms"
                            is: "TransmitAndReceiveSpacecraftSignals"
                            port in "currentStatus"
                        }
                        function "trackAndRelay" {
                            id: "trackAndRelay"
                            is: "TrackAndRelayGroundSignals"
                        }
                        function "rest" {
                            id: "rest"
                            is: "ManageCrewSchedules"
                            port in "currentStatus"
                            port out "restedStatus"
                        }
                    }
                    function "AchieveLunarOrbit" {
                        id: "AchieveLunarOrbit"
                        description: "Executes a propulsive maneuver for gravitational capture by the Moon. Corresponds to the LunarOrbitInsertionPhase."
                        port in "vehicleInLunarSphere"
                        port out "vehicleInStableLunarOrbit"
                        function "burn" {
                            id: "burn_"
                            is: "ProvideLunarOrbitInsertionThrust"
                            port in "currentStatus"
                            port out "postBurnStatus"
                        }
                        function "control" {
                            id: "control_"
                            is: "ControlLunarOrbitInsertionBurn"
                            port in "currentStatus"
                            port out "finalStatus"
                        }
                    }
                }
                function "ConductLunarOperations" {
                    id: "ConductLunarOperations"
                    description: "To perform all operations in the lunar environment, including landing, exploration, and returning to the main vehicle in lunar orbit."
                    port in "vehicleInLunarOrbit"
                    port out "returnReadyVehicle"
                    port out "samples"
                    port out "science"
                    function "PrepareForLanding" {
                        id: "PrepareForLanding"
                        description: "Prepares and separates the landing vehicle from the main vehicle. Corresponds to the LunarOrbitOperationsDescentPreparationPhase."
                        port in "combinedVehicle"
                        port out "separatedLander"
                        port out "separatedOrbiter"
                        function "checkout" {
                            id: "checkout"
                            is: "PerformOrbitalSystemCheckouts"
                            port in "currentStatus"
                            port out "checkedVehicle"
                        }
                        function "prep" {
                            id: "prep"
                            is: "PrepareLanderForSeparation"
                            port in "currentStatus"
                            port out "preppedVehicle"
                        }
                        function "undock" {
                            id: "undock"
                            is: "SeparateLanderFromOrbiter"
                            port in "currentStatus"
                            port out "lander"
                            port out "orbiter"
                        }
                    }
                    function "LandOnSurface" {
                        id: "LandOnSurface"
                        description: "Executes a controlled, powered descent from lunar orbit to the surface. Corresponds to the PoweredDescentPhase."
                        port in "landingVehicle"
                        port out "vehicleOnSurface"
                        function "burn" {
                            id: "burn__"
                            is: "ExecuteDescentBurn"
                            port in "currentStatus"
                            port out "descentStatus"
                        }
                        function "monitor" {
                            id: "monitor"
                            is: "MonitorDescent"
                            port in "currentStatus"
                            port out "monitoredStatus"
                        }
                        function "pilot" {
                            id: "pilot"
                            is: "ManuallyPilotLander"
                            port in "currentStatus"
                            port out "pilotedStatus"
                        }
                    }
                    function "ExploreLunarSurface" {
                        id: "ExploreLunarSurface"
                        description: "Conducts all planned extravehicular activities on the Moon. Corresponds to the LunarSurfaceOperationsPhase."
                        port in "vehicleOnSurface"
                        port out "collectedSamples"
                        port out "deployedScience"
                        function "postLand" {
                            id: "postLand"
                            is: "PerformPostLandingCheck"
                            port in "currentStatus"
                        }
                        function "prepEva" {
                            id: "prepEva"
                            is: "PrepareForExtravehicularActivity"
                            port in "currentStatus"
                        }
                        function "provideIndividualLifesupport" {
                            id: "provideIndividualLifesupport_"
                            is: "ProvideIndividualLifesupport"
                        }
                        function "eva" {
                            id: "eva"
                            is: "ConductSurfaceExploration"
                            port in "currentStatus"
                        }
                        function "collect" {
                            id: "collect"
                            is: "GatherLunarSamples"
                            port out "samples"
                        }
                        function "deploy" {
                            id: "deploy"
                            is: "DeploySurfaceExperiments"
                            port out "scienceData"
                        }
                        function "conclude" {
                            id: "conclude"
                            is: "CompleteExtravehicularActivity"
                            port in "currentStatus"
                        }
                        function "rest" {
                            id: "rest_"
                            is: "ConductSurfaceRestPeriod"
                        }
                    }
                    function "ReturnToOrbitAndDock" {
                        id: "ReturnToOrbitAndDock"
                        description: "Leaves the lunar surface, achieves orbit, and physically connects with the primary vehicle. Corresponds to the LunarAscentRendezvousPhase."
                        port in "ascentVehicleOnSurface"
                        port in "mainVehicleInOrbit"
                        port in "samplesToTransfer"
                        port out "mainVehicleWithCrew"
                        function "ascend" {
                            id: "ascend"
                            is: "ExecuteAscentBurn"
                            port in "currentStatus"
                            port out "ascentVehicle"
                        }
                        function "calculate" {
                            id: "calculate_"
                            is: "CalculateRendezvousManeuvers"
                            port in "currentStatus"
                            port out "rendezvousPlan"
                        }
                        function "rendezvous" {
                            id: "rendezvous"
                            is: "ExecuteRendezvousBurns"
                            port in "rendezvousPlan"
                            port out "proximalVehicle"
                        }
                        function "dock" {
                            id: "dock"
                            is: "ConnectSpacecraft"
                            port in "currentStatus"
                            port out "dockedVehicle"
                        }
                        function "transfer" {
                            id: "transfer"
                            is: "TransferCrewAndMaterials"
                            port in "currentStatus"
                            port in "samples"
                            port out "postTransferVehicle"
                        }
                        function "jettison" {
                            id: "jettison"
                            is: "DiscardAscentStage"
                            port in "currentStatus"
                            port out "finalVehicle"
                        }
                    }
                }
                function "ExecuteReturnJourney" {
                    id: "ExecuteReturnJourney"
                    description: "To execute all operations required to leave lunar orbit, travel back to Earth, and safely recover the crew and mission assets."
                    port in "returnReadyVehicle"
                    port in "samples"
                    port out "recoveredCrew"
                    port out "recoveredSamples"
                    function "SetTransearthCourse" {
                        id: "SetTransearthCourse"
                        description: "Executes a propulsive maneuver to leave lunar orbit and begin the journey to Earth. Corresponds to the TransEarthInjectionPhase."
                        port in "mainVehicleInLunarOrbit"
                        port out "vehicleOnTransearthTrajectory"
                        function "burn" {
                            id: "burn___"
                            is: "ExecuteTransearthBurn"
                            port in "currentStatus"
                            port out "finalStatus"
                        }
                    }
                    function "TraverseToEarth" {
                        id: "TraverseToEarth"
                        description: "Navigates the vehicle to Earth and prepares for reentry. Corresponds to the TransEarthCoastPhase."
                        port in "vehicleOnTransearthTrajectory"
                        port out "reentryReadyVehicle"
                        function "monitorSystems" {
                            id: "monitorSystems"
                            is: "MonitorOnboardSystems"
                            port in "currentStatus"
                        }
                        function "monitorTelemetry" {
                            id: "monitorTelemetry"
                            is: "MonitorDownlinkedTelemetry"
                            port in "currentStatus"
                        }
                        function "transmitAndReceive" {
                            id: "transmitAndReceive"
                            is: "TransmitAndReceiveSpacecraftSignals"
                            port in "currentStatus"
                        }
                        function "trackAndRelay" {
                            id: "trackAndRelay_"
                            is: "TrackAndRelayGroundSignals"
                        }
                        function "prep" {
                            id: "prep_"
                            is: "PrepareVehicleForReentry"
                            port in "currentStatus"
                            port out "finalStatus"
                        }
                    }
                    function "ReenterAndLand" {
                        id: "ReenterAndLand"
                        description: "Manages atmospheric entry and executes a safe landing. Corresponds to the ReentryLandingPhase."
                        port in "reentryReadyVehicle"
                        port out "landedVehicle"
                        function "separate" {
                            id: "separate"
                            is: "SeparateReturnCapsule"
                            port in "currentStatus"
                            port out "capsule"
                        }
                        function "guide" {
                            id: "guide_"
                            is: "GuideAtmosphericEntry"
                            port in "currentStatus"
                            port out "reentryStatus"
                        }
                        function "deploy" {
                            id: "deploy_"
                            is: "DeployDecelerators"
                            port in "currentStatus"
                            port out "descentStatus"
                        }
                        function "splashdown" {
                            id: "splashdown"
                            is: "PerformSplashdown"
                            port in "currentStatus"
                            port out "finalStatus"
                        }
                    }
                    function "RecoverCrewAndAssets" {
                        id: "RecoverCrewAndAssets"
                        description: "Retrieves the crew and vehicle and initiates post-mission protocols. Corresponds to the RecoveryQuarantinePhase."
                        port in "landedVehicle"
                        port in "samplesOnboard"
                        port out "securedCrew"
                        port out "securedSamples"
                        function "retrieve" {
                            id: "retrieve"
                            is: "RetrieveVehicleAndCrew"
                            port in "currentStatus"
                            port out "recoveredCrew"
                            port out "recoveredVehicle"
                        }
                        function "quarantine" {
                            id: "quarantine"
                            is: "ExecuteQuarantine"
                            port in "crew"
                        }
                        function "secure" {
                            id: "secure"
                            is: "SecureReturnedSamples"
                            port in "samples"
                            port out "finalSamples"
                        }
                    }
                }
            }
            component "Neil Armstrong" {
                id: "Neil Armstrong"
                description: "Neil Armstrong was the Commander of Apollo 11. A veteran of the Gemini program and a skilled test pilot, he became the first human to walk on the lunar surface on July 20, 1969."
                dateOfBirth: "new Iso8601DateTime(\"1930-08-05\")"
                dateOfDeath: "new Iso8601DateTime(\"2012-08-25\")"
            }
            component "Michael Collins" {
                id: "Michael Collins"
                description: "Michael Collins was the Command Module Pilot for Apollo 11. He remained in lunar orbit aboard the Command Module \"Columbia\" while his crewmates explored the surface, a critical role for their safe return."
                dateOfBirth: "new Iso8601DateTime(\"1930-10-31\")"
                dateOfDeath: "new Iso8601DateTime(\"2021-04-28\")"
            }
            component "Buzz Aldrin" {
                id: "Buzz Aldrin"
                description: "Edwin \"Buzz\" Aldrin was the Lunar Module Pilot for Apollo 11. He joined Neil Armstrong on the lunar surface, becoming the second human to walk on the Moon."
                dateOfBirth: "new Iso8601DateTime(\"1930-01-20\")"
            }
            component "ProgrammaticAndPublicAffairsManagement" {
                id: "ProgrammaticAndPublicAffairsManagement"
                description: "The ability to manage budgets, schedules, stakeholder communications, and public relations to maintain political and public support for a large-scale, high-cost program."
            }
            component "HeavyLiftLaunch" {
                id: "HeavyLiftLaunch"
                description: "The capability to propel extremely heavy payloads from Earth's surface into various orbital and interplanetary trajectories."
            }
            component "MultiStagePropulsion" {
                id: "MultiStagePropulsion"
                description: "The ability to efficiently shed mass during ascent through the sequential operation of multiple rocket stages, optimizing for different phases of flight."
            }
            component "PrecisionOrbitalMechanics" {
                id: "PrecisionOrbitalMechanics"
                description: "The capability to accurately calculate, execute, and refine trajectories and orbital maneuvers in complex gravitational fields (Earth-Moon system). This includes achieving and maintaining stable orbits, as well as performing precise rendezvous and docking operations in space."
            }
            component "AtmosphericReentryAndLanding" {
                id: "AtmosphericReentryAndLanding"
                description: "The ability to guide a spacecraft safely through Earth's atmosphere, dissipate extreme heat, control descent, and achieve a soft water landing."
            }
            component "DeepSpaceHabitationAndLifeSupport" {
                id: "DeepSpaceHabitationAndLifeSupport"
                description: "Providing sustained environments for humans in the vacuum of space, managing air quality, temperature, humidity, water, and waste for extended durations."
            }
            component "AutonomousAndAssistedNavigationGuidanceAndControl" {
                id: "AutonomousAndAssistedNavigationGuidanceAndControl"
                description: "Systems capable of determining position and velocity, calculating optimal paths, and executing maneuvers, both independently and with ground assistance."
            }
            component "InSpacePropulsion" {
                id: "InSpacePropulsion"
                description: "The ability to provide thrust for major orbital changes and trans-planetary injections using highly reliable and powerful engines."
            }
            component "PowerGenerationAndDistribution" {
                id: "PowerGenerationAndDistribution"
                description: "Reliable systems to generate and distribute electrical power to all spacecraft systems throughout the mission."
            }
            component "LunarSurfaceLandingAndAscent" {
                id: "LunarSurfaceLandingAndAscent"
                description: "The specialized capability to de-orbit, perform a powered descent to a planetary surface, and then launch from that surface back into orbit."
            }
            component "GlobalTrackingAndCommunication" {
                id: "GlobalTrackingAndCommunication"
                description: "Maintaining continuous contact and data exchange with spacecraft across vast distances using a worldwide network of ground stations."
            }
            component "RealtimeMissionControl" {
                id: "RealtimeMissionControl"
                description: "The ability to monitor thousands of operational parameters, analyze data, and make critical, time-sensitive decisions to guide and support the mission."
            }
            component "ContingencyPlanningAndRapidResponse" {
                id: "ContingencyPlanningAndRapidResponse"
                description: "Developing and implementing procedures to address unforeseen anomalies or failures swiftly and effectively."
            }
            component "CrewTrainingAndSimulation" {
                id: "CrewTrainingAndSimulation"
                description: "Rigorous programs to prepare astronauts for all mission aspects, including emergency procedures, through realistic simulations and drills."
            }
            component "HumanPerformanceUnderExtremeStress" {
                id: "HumanPerformanceUnderExtremeStress"
                description: "The ability of individuals and teams to maintain focus, make sound judgments, and execute complex tasks during high-stakes situations with immense pressure."
            }
            component "ComplexSoftwareDevelopment" {
                id: "ComplexSoftwareDevelopment"
                description: "Creating and verifying intricate software for real-time control, navigation, and system management in highly constrained environments."
            }
            component "ComplexLogisticsAndIntegration" {
                id: "ComplexLogisticsAndIntegration"
                description: "The capacity to manage the design, manufacturing, assembly, and testing of highly complex, interdependent systems on an unprecedented scale."
            }
            component "LaunchOperationsManagement" {
                id: "LaunchOperationsManagement"
                description: "The specialized ability to meticulously prepare, fuel, and execute the launch of large-scale rockets with precision and safety."
            }
            component "AdaptabilityAndProblemSolving" {
                id: "AdaptabilityAndProblemSolving"
                description: "The inherent human capacity to adjust to unexpected challenges and devise creative solutions in real-time."
            }
            component "ExtravehicularActivitySupport" {
                id: "ExtravehicularActivitySupport"
                description: "Providing specialized protective suits and equipment that allow humans to operate and perform tasks safely and effectively in the vacuum and harsh environments of space and planetary surfaces."
            }
            component "LunarMobility" {
                id: "LunarMobility"
                description: "The ability for astronauts to move and work efficiently in lunar gravity."
            }
            component "ScientificDataAcquisitionAndSampleReturn" {
                id: "ScientificDataAcquisitionAndSampleReturn"
                description: "The capability to deploy scientific instruments, collect geological samples from an extraterrestrial body, and return them safely for Earth-based analysis."
            }
            component "PlanetaryProtection" {
                id: "PlanetaryProtection"
                description: "Implementing stringent protocols to prevent the biological contamination of Earth from returned extraterrestrial samples and to minimize contamination of the extraterrestrial body."
            }
            component "RobustCommunicationAndTelemetry" {
                id: "RobustCommunicationAndTelemetry"
                description: "Establishing and maintaining reliable data and voice links over vast distances, including transmitting live video."
            }
            component "ToolDevelopment" {
                id: "ToolDevelopment"
                description: "Creating specialized tools for lunar surface tasks (e.g., sample collection, instrument deployment)."
            }
        }
        component "as-201" {
            id: "as-201"
            description: "First flight of Saturn IB and Block I CSM; suborbital to Atlantic Ocean; qualified heat shield to orbital reentry speed."
            is: "Mission"
        }
        component "as-203" {
            id: "as-203"
            description: "No spacecraft; observations of liquid hydrogen fuel behavior in orbit to support design of S-IVB restart capability."
            is: "Mission"
        }
        component "as-202" {
            id: "as-202"
            description: "Suborbital flight of CSM to Pacific Ocean."
            is: "Mission"
        }
        component "apollo1" {
            id: "apollo1"
            description: "Not flown. All crew members died in a fire during a launch pad test on January 27, 1967."
            is: "MannedSpaceMission"
            component "Gus Grissom" {
                id: "Gus Grissom"
                description: "Virgil \"Gus\" Grissom was one of the original Mercury Seven astronauts. He was the second American in space and the first person to fly in space twice. He was the commander of the ill-fated Apollo 1 mission and tragically died in the launch pad fire."
                dateOfBirth: "new Iso8601DateTime(\"1926-04-03\")"
                dateOfDeath: "new Iso8601DateTime(\"1967-01-27\")"
            }
            component "Ed White" {
                id: "Ed White"
                description: "Edward H. White II was the first American to perform a spacewalk, during the Gemini 4 mission. As Senior Pilot for the Apollo 1 mission, he died in the launch pad fire alongside his crewmates."
                dateOfBirth: "new Iso8601DateTime(\"1930-11-14\")"
                dateOfDeath: "new Iso8601DateTime(\"1967-01-27\")"
            }
            component "Roger B. Chaffee" {
                id: "Roger B. Chaffee"
                description: "A rookie astronaut selected for the first crewed low Earth orbital test of the Apollo spacecraft. He served as the Pilot for Apollo 1 and tragically died in the launch pad fire with his crew."
                dateOfBirth: "new Iso8601DateTime(\"1935-02-15\")"
                dateOfDeath: "new Iso8601DateTime(\"1967-01-27\")"
            }
        }
        component "apollo4" {
            id: "apollo4"
            description: "First test flight of Saturn V, placed a CSM in a high Earth orbit; demonstrated S-IVB restart; qualified CM heat shield to lunar reentry speed."
            is: "Mission"
        }
        component "apollo5" {
            id: "apollo5"
            description: "Earth orbital flight test of LM, launched on Saturn IB; demonstrated ascent and descent propulsion; human-rated the LM. No crew."
            is: "Mission"
        }
        component "apollo6" {
            id: "apollo6"
            description: "Uncrewed, second flight of Saturn V, attempted demonstration of trans-lunar injection, and direct-return abort using SM engine; three engine failures, including failure of S-IVB restart. Flight controllers used SM engine to repeat Apollo 4's flight profile. Human-rated the Saturn V."
            is: "Mission"
        }
        component "apollo7" {
            id: "apollo7"
            description: "First crewed Earth orbital demonstration of Block II CSM, launched on Saturn IB. First live television broadcast from a crewed mission."
            is: "MannedSpaceMission"
            component "Wally Schirra" {
                id: "Wally Schirra"
                description: "Walter \"Wally\" Schirra was the only astronaut to fly in all three of America's first human spaceflight programs: Mercury, Gemini, and Apollo. He commanded Apollo 7, the first crewed flight of the Apollo program."
                dateOfBirth: "new Iso8601DateTime(\"1923-03-12\")"
                dateOfDeath: "new Iso8601DateTime(\"2007-05-03\")"
            }
            component "Walt Cunningham" {
                id: "Walt Cunningham"
                description: "Walter Cunningham was a physicist and the Lunar Module Pilot on Apollo 7. This mission was a critical 11-day Earth-orbital test of the Apollo Command/Service Module, proving its space-worthiness."
                dateOfBirth: "new Iso8601DateTime(\"1932-03-16\")"
                dateOfDeath: "new Iso8601DateTime(\"2023-01-03\")"
            }
            component "Donn Eisele" {
                id: "Donn Eisele"
                description: "Donn F. Eisele was the Command Module Pilot on Apollo 7, the first crewed flight of the Apollo spacecraft, which successfully tested the vehicle in Earth orbit."
                dateOfBirth: "new Iso8601DateTime(\"1930-06-23\")"
                dateOfDeath: "new Iso8601DateTime(\"1987-12-02\")"
            }
        }
        component "apollo8" {
            id: "apollo8"
            description: "First crewed flight of Saturn V; First crewed flight to Moon; CSM made 10 lunar orbits in 20 hours."
            is: "MannedSpaceMission"
            component "Frank Borman" {
                id: "Frank Borman"
                description: "Frank Borman was the Commander of the Apollo 8 mission, the first crewed flight to orbit the Moon. This historic 1968 mission was a crucial step towards the lunar landing."
                dateOfBirth: "new Iso8601DateTime(\"1928-03-14\")"
                dateOfDeath: "new Iso8601DateTime(\"2023-11-07\")"
            }
            component "James Lovell" {
                id: "James Lovell_"
                is: "James Lovell"
            }
            component "William Anders" {
                id: "William Anders"
                description: "William \"Bill\" Anders was the Lunar Module Pilot on Apollo 8. He is famous for taking the iconic \"Earthrise\" photograph, showing the Earth rising above the lunar horizon."
                dateOfBirth: "new Iso8601DateTime(\"1933-10-17\")"
                dateOfDeath: "new Iso8601DateTime(\"2024-06-07\")"
            }
        }
        component "apollo9" {
            id: "apollo9"
            description: "Second crewed flight of Saturn V; First crewed flight of CSM and LM in Earth orbit; demonstrated portable life support system to be used on the lunar surface."
            is: "MannedSpaceMission"
            component "James McDivitt" {
                id: "James McDivitt"
                description: "James McDivitt was the Commander of Gemini 4 and Apollo 9 (first flight of the Lunar Module)."
                dateOfBirth: "new Iso8601DateTime(\"1929-06-10\")"
                dateOfDeath: "new Iso8601DateTime(\"2022-10-13\")"
            }
            component "David Scott" {
                id: "David Scott_"
                is: "David Scott"
            }
            component "Russell Schweickart" {
                id: "Russell Schweickart"
                description: "Russell \"Rusty\" Schweickart was the Lunar Module Pilot on Apollo 9. He performed the first extravehicular activity (EVA) of the Apollo program, testing the new spacesuit's life support system."
                dateOfBirth: "new Iso8601DateTime(\"1935-10-25\")"
            }
        }
        component "apollo10" {
            id: "apollo10"
            description: "Dress rehearsal for first lunar landing; flew LM down to 50,000 ft (15 km; 9.5 mi) from lunar surface."
            is: "MannedSpaceMission"
            component "Thomas Stafford" {
                id: "Thomas Stafford"
                description: "Thomas Stafford was the Commander of Apollo 10, the final \"dress rehearsal\" mission that flew to the Moon and tested the Lunar Module in lunar orbit, descending to within 9 miles of the surface."
                dateOfBirth: "new Iso8601DateTime(\"1930-09-17\")"
                dateOfDeath: "new Iso8601DateTime(\"2024-03-18\")"
            }
            component "John Young" {
                id: "John Young_"
                is: "John Young"
            }
            component "Eugene Cernan" {
                id: "Eugene Cernan_"
                is: "Eugene Cernan"
            }
        }
        component "apollo12" {
            id: "apollo12"
            description: "Second landing, in Ocean of Storms near Surveyor 3. Surface EVA time: 7h 45m. Samples returned: 75.62 lb (34.30 kg)."
            is: "MannedSpaceMission"
            component "Pete Conrad" {
                id: "Pete Conrad"
                description: "Charles \"Pete\" Conrad Jr. was the Commander of Apollo 12, the second lunar landing mission. He was the third person to walk on the Moon."
                dateOfBirth: "new Iso8601DateTime(\"1930-06-02\")"
                dateOfDeath: "new Iso8601DateTime(\"1999-07-08\")"
            }
            component "Richard Gordon" {
                id: "Richard Gordon"
                description: "Richard \"Dick\" Gordon Jr. was the Command Module Pilot for Apollo 12, orbiting the Moon while his crewmates landed in the Ocean of Storms."
                dateOfBirth: "new Iso8601DateTime(\"1929-10-05\")"
                dateOfDeath: "new Iso8601DateTime(\"2017-11-06\")"
            }
            component "Alan Bean" {
                id: "Alan Bean"
                description: "Alan Bean was the Lunar Module Pilot for Apollo 12, becoming the fourth person to walk on the Moon. After retiring from NASA, he became a renowned artist, painting scenes from the Apollo missions."
                dateOfBirth: "new Iso8601DateTime(\"1932-03-15\")"
                dateOfDeath: "new Iso8601DateTime(\"2018-05-26\")"
            }
        }
        component "apollo13" {
            id: "apollo13"
            description: "Third landing attempt aborted in transit to the Moon, due to SM failure. Crew used LM as \"lifeboat\" to return to Earth. Mission called a \"successful failure\"."
            is: "MannedSpaceMission"
            component "James Lovell" {
                id: "James Lovell__"
                is: "James Lovell"
            }
            component "Jack Swigert" {
                id: "Jack Swigert"
                description: "John \"Jack\" Swigert Jr. was the Command Module Pilot for the dramatic Apollo 13 mission. He was a late replacement for Ken Mattingly and played a key role in the crew's safe return."
                dateOfBirth: "new Iso8601DateTime(\"1931-08-30\")"
                dateOfDeath: "new Iso8601DateTime(\"1982-12-27\")"
            }
            component "Fred Haise" {
                id: "Fred Haise"
                description: "Fred Haise was the Lunar Module Pilot for Apollo 13. He was scheduled to be the sixth person to walk on the Moon before an in-flight emergency forced the mission to abort its landing."
                dateOfBirth: "new Iso8601DateTime(\"1933-11-14\")"
            }
        }
        component "apollo14" {
            id: "apollo14"
            description: "Third landing, in Fra Mauro formation. Surface EVA time: 9h 21m. Samples returned: 94.35 lb (42.80 kg)."
            is: "MannedSpaceMission"
            component "Alan Shepard" {
                id: "Alan Shepard"
                description: "Alan Shepard was one of the original Mercury Seven and the first American in space. After overcoming an inner-ear disorder, he commanded Apollo 14, becoming the fifth person to walk on the Moon, where he famously hit two golf balls."
                dateOfBirth: "new Iso8601DateTime(\"1923-11-18\")"
                dateOfDeath: "new Iso8601DateTime(\"1998-07-21\")"
            }
            component "Stuart Roosa" {
                id: "Stuart Roosa"
                description: "Stuart Roosa was the Command Module Pilot for Apollo 14. He carried hundreds of tree seeds into lunar orbit as part of a joint project with the U.S. Forest Service."
                dateOfBirth: "new Iso8601DateTime(\"1933-08-16\")"
                dateOfDeath: "new Iso8601DateTime(\"1994-12-12\")"
            }
            component "Edgar Mitchell" {
                id: "Edgar Mitchell"
                description: "Edgar Mitchell was the Lunar Module Pilot for Apollo 14 and the sixth person to walk on the Moon, exploring the Fra Mauro formation."
                dateOfBirth: "new Iso8601DateTime(\"1930-09-17\")"
                dateOfDeath: "new Iso8601DateTime(\"2016-02-04\")"
            }
        }
        component "apollo15" {
            id: "apollo15"
            description: "Fourth landing, in Hadley-Apennine. First extended mission, used Rover on Moon. Surface EVA time: 18h 33m. Samples returned: 169.10 lb (76.70 kg)."
            is: "MannedSpaceMission"
            component "David Scott" {
                id: "David Scott__"
                is: "David Scott"
            }
            component "Alfred Worden" {
                id: "Alfred Worden"
                description: "Alfred Worden was the Command Module Pilot for Apollo 15. During the return journey from the Moon, he performed the first-ever deep-space extravehicular activity (EVA)."
                dateOfBirth: "new Iso8601DateTime(\"1932-02-07\")"
                dateOfDeath: "new Iso8601DateTime(\"2020-03-18\")"
            }
            component "James Irwin" {
                id: "James Irwin"
                description: "James Irwin was the Lunar Module Pilot for Apollo 15 and the eighth person to walk on the Moon. He was one of the first to explore the lunar mountains and drive the Lunar Roving Vehicle."
                dateOfBirth: "new Iso8601DateTime(\"1930-03-17\")"
                dateOfDeath: "new Iso8601DateTime(\"1991-08-08\")"
            }
        }
        component "apollo16" {
            id: "apollo16"
            description: "Fifth landing, in Plain of Descartes. Second extended mission, used Rover on Moon. Surface EVA time: 20h 14m. Samples returned: 207.89 lb (94.30 kg)."
            is: "MannedSpaceMission"
            component "John Young" {
                id: "John Young__"
                is: "John Young"
            }
            component "Ken Mattingly" {
                id: "Ken Mattingly"
                description: "Thomas Kenneth \"Ken\" Mattingly II was famously removed from the Apollo 13 crew due to exposure to measles but later served as the Command Module Pilot for Apollo 16, orbiting the Moon during the mission."
                dateOfBirth: "new Iso8601DateTime(\"1936-03-17\")"
                dateOfDeath: "new Iso8601DateTime(\"2023-10-31\")"
            }
            component "Charles Duke" {
                id: "Charles Duke"
                description: "Charles Duke served as the CAPCOM for Apollo 11, and his voice was heard by millions during the landing. He later flew as the Lunar Module Pilot for Apollo 16, becoming the tenth person to walk on the Moon."
                dateOfBirth: "new Iso8601DateTime(\"1935-10-03\")"
            }
        }
        component "apollo17" {
            id: "apollo17"
            description: "Only Saturn V night launch. Sixth landing, in Taurus-Littrow. Third extended mission, used Rover on Moon. First geologist on the Moon. Apollo's last crewed Moon landing. Surface EVA time: 22h 2m. Samples returned: 243.40 lb (110.40 kg)."
            is: "MannedSpaceMission"
            component "Eugene Cernan" {
                id: "Eugene Cernan__"
                is: "Eugene Cernan"
            }
            component "Ronald Evans" {
                id: "Ronald Evans"
                description: "Ronald Evans was the Command Module Pilot for Apollo 17. During the return flight from the Moon, he performed an EVA to retrieve film cassettes from the Service Module, which was the last deep-space EVA of the Apollo program."
                dateOfBirth: "new Iso8601DateTime(\"1933-11-10\")"
                dateOfDeath: "new Iso8601DateTime(\"1990-04-07\")"
            }
            component "Harrison Schmitt" {
                id: "Harrison Schmitt"
                description: "Harrison \"Jack\" Schmitt was the Lunar Module Pilot for Apollo 17. As a professional geologist, he was the only scientist to walk on the Moon and the twelfth person to do so."
                dateOfBirth: "new Iso8601DateTime(\"1935-07-03\")"
            }
        }
    }
    component "EarthEnvironment" {
        id: "EarthEnvironment"
        description: "The starting and ending physical environment for the mission."
        surfaceGravity: "1 gn"
        standardAtmosphericPressure: 101325 Pa
        rotationalVelocity: 465 m/s
        reentryInterfaceAltitude: 122 km
    }
    component "CislunarSpaceEnvironment" {
        id: "CislunarSpaceEnvironment"
        description: "The physical environment between Earth and the Moon."
        ambientPressure: 0.000000000000032 Pa
        solarIrradiance: "1361 W/m^2"
        backgroundTemperature: 2.7 K
    }
    component "LunarEnvironment" {
        id: "LunarEnvironment"
        description: "The destination physical environment."
        surfaceGravity: "1.62 m⋅s⁻²"
        surfacePressure: 0.0000000003 Pa
        maxDayTemperature: 400 K
        minNightTemperature: 100 K
    }
    component "LaunchSite" {
        id: "LaunchSite"
        description: "The specific ground facility from which the mission originates."
        location: "Merritt Island, Florida, USA"
        geographicCoordinates: "28.57° N, 80.65° W"
        component "vehicleAssemblyBuilding" {
            id: "vehicleAssemblyBuilding_"
            is: "VehicleAssemblyBuilding"
        }
        component "launchControlCenter" {
            id: "launchControlCenter_"
            is: "LaunchControlCenter"
        }
        component "crawlerTransporter" {
            id: "crawlerTransporter_"
            is: "CrawlerTransporter"
        }
    }
    component "SaturnV" {
        id: "SaturnV"
        description: "The three-stage, super heavy-lift launch vehicle used to send the Apollo spacecraft to the Moon."
        height: 110.6 m
        launchMass: 2970000 kg
        payloadToLunarTransferOrbit: 43500 kg
        component "S-IC" {
            id: "S-IC"
            description: "The first stage of the Saturn V, providing the initial ~7.7 million pounds of thrust for liftoff."
            propellantMass: 2077000 kg
            dryMass: 137000 kg
            port out "upperStagePort"
            component "engine1" {
                id: "engine1"
                is: "F-1"
            }
            component "engine2" {
                id: "engine2"
                is: "F-1"
            }
            component "engine3" {
                id: "engine3"
                is: "F-1"
            }
            component "engine4" {
                id: "engine4"
                is: "F-1"
            }
            component "engine5" {
                id: "engine5"
                is: "F-1"
            }
            component "engines" {
                id: "engines_"
            }
        }
        component "S-II" {
            id: "S-II"
            description: "The second stage of the Saturn V, providing massive thrust in the upper atmosphere and vacuum of space."
            propellantMass: 443000 kg
            dryMass: 36200 kg
            port in "lowerStagePort"
            port out "upperStagePort"
            component "engine1" {
                id: "engine1_"
                is: "J-2"
            }
            component "engine2" {
                id: "engine2_"
                is: "J-2"
            }
            component "engine3" {
                id: "engine3_"
                is: "J-2"
            }
            component "engine4" {
                id: "engine4_"
                is: "J-2"
            }
            component "engine5" {
                id: "engine5_"
                is: "J-2"
            }
            component "engines" {
                id: "engines__"
            }
        }
        component "S-IVB" {
            id: "S-IVB"
            description: "The third stage of the Saturn V, responsible for the final push into Earth orbit and the critical Translunar Injection burn."
            propellantMass: 109500 kg
            dryMass: 13500 kg
            port in "lowerStagePort"
            port inout "controlPort"
            component "engine" {
                id: "engine"
                is: "J-2"
            }
            component "engines" {
                id: "engines___"
            }
        }
        component "SaturnVInstrumentUnit" {
            id: "SaturnVInstrumentUnit"
            description: "The 'brain' of the Saturn V, providing guidance and control during ascent."
            dryMass: 1950 kg
            failureRate: "5E-6 1/h"
            powerLoad: 450 W
            port inout "stageControlPort"
            port inout "payloadInterfacePort"
        }
        component "stages" {
            id: "stages_"
        }
    }
    component "SA-500F" {
        id: "SA-500F"
        description: "Facilities integration. Used to check precise fits and test facilities operation on Pad 39A before a flight model was ready. First stage scrapped, second stage converted to S-II-F/D, third stage on display at Kennedy Space Center."
    }
    component "SA-500D" {
        id: "SA-500D"
        description: "Dynamic testing. Used to evaluate the vehicle's response to vibrations. On display at the U.S. Space & Rocket Center, Huntsville, Alabama."
    }
    component "S-IC-T" {
        id: "S-IC-T"
        description: "All Systems Test. First stage used for static test firing at Marshall Space Flight Center. On display at Kennedy Space Center."
    }
    component "SA-501" {
        id: "SA-501"
        description: "Apollo 4 First uncrewed, all-up test flight."
    }
    component "SA-502" {
        id: "SA-502"
        description: "Apollo 6. Second uncrewed test flight; J-2 engine problems caused early shutdown of two engines in second stage, and prevented third stage restart."
    }
    component "SA-503" {
        id: "SA-503"
        description: "Apollo 8. First crewed flight; first trans-lunar injection of Apollo command and service module."
    }
    component "SA-504" {
        id: "SA-504"
        description: "Apollo 9. Crewed low Earth orbit test of complete Apollo spacecraft with the Lunar Module (LM)."
    }
    component "SA-505" {
        id: "SA-505"
        description: "Apollo 10. Second crewed trans-lunar injection of complete Apollo spacecraft with LM; Only Saturn V launched from Pad 39B."
    }
    component "SA-507" {
        id: "SA-507"
        description: "Apollo 12. Vehicle was struck twice by lightning shortly after liftoff, but did not suffer serious damage. Made second crewed lunar landing, near Surveyor 3 at Ocean of Storms"
    }
    component "SA-508" {
        id: "SA-508"
        description: "Apollo 13. Severe pogo oscillations in second stage caused early center engine shutdown; guidance compensated by burning remaining engines longer. Lunar landing mission was aborted by service module failure."
    }
    component "SA-509" {
        id: "SA-509"
        description: "Apollo 14. Third crewed lunar landing, near Fra Mauro, Apollo 13's intended landing site."
    }
    component "SA-510" {
        id: "SA-510"
        description: "Apollo 15. Fourth crewed lunar landing, at Hadley-Apennine. First extended Apollo mission, carrying lunar orbital Scientific Instrument Module and Lunar Roving Vehicle."
    }
    component "SA-511" {
        id: "SA-511"
        description: "Apollo 16. Fifth crewed lunar landing, at Descartes Highlands."
    }
    component "SA-512" {
        id: "SA-512"
        description: "Apollo 17. Only night launch. Sixth and final crewed lunar landing, at Taurus-Littrow."
    }
    component "SA-513" {
        id: "SA-513"
        description: "Skylab 1. Uncrewed launch of the Skylab orbital workshop, which replaced the third stage, S-IVB-513, on display at Johnson Space Center. Originally designated for canceled Apollo 18."
    }
    component "SA-514" {
        id: "SA-514"
        description: "Unused. Originally designated for canceled Apollo 18 or 19; never used. It was proposed for launching an International Skylab. This station would have been serviced by Apollo, Soyuz and later by the Space Shuttle. The first stage (S-IC-14) on display at Johnson Space Center, second and third stage (S-II-14, S-IV-14) on display at Kennedy Space Center. The S-II interstage is located at Parque de las Ciencias in Puerto Rico."
    }
    component "SA-515" {
        id: "SA-515"
        description: "Unused. Originally designated for Apollo 20, never used. Later it was proposed to launch the backup Skylab station into orbit sometime between January 1975 and April 1976. That way, it could expand the Apollo-Soyuz mission by 56-90 days. The first stage was on display at Michoud Assembly Facility, until June 2016 then was moved to the INFINITY Science Center in Mississippi. The second stage (S-II-15) is on display at Johnson Space Center. The third stage was converted to a backup Skylab orbital workshop and is on display at the National Air and Space Museum."
    }
    component "CSM-098" {
        id: "CSM-098"
        description: "2TV-1 Used in thermal vacuum tests. CSM on display at Academy of Science Museum, Moscow, Russia as individual part def of the Apollo Soyuz Test Project display."
    }
    component "CM-099" {
        id: "CM-099"
        description: "2S-1 Skylab flight crew interface training; impact tests. Scrapped"
    }
    component "CSM-100" {
        id: "CSM-100"
        description: "2S-2 static structural testing. Command module \"transferred to Smithsonian as an artifact\", service module on display at New Mexico Museum of Space History"
    }
    component "CSM-101" {
        id: "CSM-101"
        description: "Apollo 7. Command module was on display at National Museum of Science and Technology, Ottawa, Ontario, Canada from 1974 until 2004, now at the Frontiers of Flight Museum, Dallas, Texas after 30 years of being on loan."
    }
    component "CSM-102" {
        id: "CSM-102"
        description: "Launch Complex 34 checkout vehicle. Command module scrapped; service module is at JSC on top of the Little Joe II in Rocket Park with Boiler Plate 22 command module."
    }
    component "CSM-103" {
        id: "CSM-103"
        description: "Apollo 8. Command module on display at the Museum of Science and Industry in Chicago"
    }
    component "CSM-104" {
        id: "CSM-104"
        description: "Gumdrop\tApollo 9. Command module on display at San Diego Air and Space Museum"
    }
    component "CSM-105" {
        id: "CSM-105"
        description: "acoustic tests\tOn display at National Air and Space Museum, Washington, D.C. as individual part def of the Apollo Soyuz Test Project display."
    }
    component "CSM-106" {
        id: "CSM-106"
        description: "Charlie Brown\tApollo 10. Command module on display at Science Museum, London"
    }
    component "CSM-108" {
        id: "CSM-108"
        description: "Yankee Clipper\tApollo 12. Command module on display at Virginia Air & Space Center, Hampton, Virginia; previously on display at the National Naval Aviation Museum at Naval Air Station Pensacola, Pensacola, Florida (exchanged for CSM-116)"
    }
    component "CSM-109" {
        id: "CSM-109"
        description: "Odyssey\tApollo 13. Command module on display at Kansas Cosmosphere and Space Center"
    }
    component "CSM-110" {
        id: "CSM-110"
        description: "Kitty Hawk\tApollo 14. Command module on display at the Kennedy Space Center"
    }
    component "CSM-111" {
        id: "CSM-111"
        description: "Apollo Soyuz Test Project. Command module currently on display at California Science Center in Los Angeles, California (formerly displayed at the Kennedy Space Center Visitor Complex)"
    }
    component "CSM-112" {
        id: "CSM-112"
        description: "Endeavour\tApollo 15. Command module on display at National Museum of the United States Air Force, Wright-Patterson Air Force Base, Dayton, Ohio"
    }
    component "CSM-113" {
        id: "CSM-113"
        description: "Casper\tApollo 16. Command module on display at U.S. Space & Rocket Center, Huntsville, Alabama"
    }
    component "CSM-114" {
        id: "CSM-114"
        description: "America\tApollo 17. Command module on display at Space Center Houston, Houston, Texas"
    }
    component "CSM-115" {
        id: "CSM-115"
        description: "Apollo 19. Never fully completed - service module does not have its SPS nozzle installed. On display as individual part def of the Saturn V display at Johnson Space Center, Houston, Texas"
    }
    component "CSM-115a" {
        id: "CSM-115a"
        description: "Apollo 20. Never fully completed - internal structures not installed, used for spare individual part defs. Remaining items sent to Japan for exhibition in 1978 and never returned, possibly scrapped."
    }
    component "CSM-116" {
        id: "CSM-116"
        description: "Skylab 2. Command module on display at National Museum of Naval Aviation, Naval Air Station Pensacola, Pensacola, Florida"
    }
    component "CSM-117" {
        id: "CSM-117"
        description: "Skylab 3. Command module on display at Great Lakes Science Center, current location of the NASA Glenn Research Center Visitor Center, Cleveland, Ohio"
    }
    component "CSM-118" {
        id: "CSM-118"
        description: "Skylab 4.\tCommand module on display at Oklahoma History Center (formerly displayed at the National Air and Space Museum, Washington, D.C.)"
    }
    component "CSM-119" {
        id: "CSM-119"
        description: "Skylab Rescue and ASTP backup.\tOn display at the Kennedy Space Center"
    }
    component "LTA-1" {
        id: "LTA-1"
        description: "Not flown\tCradle of Aviation Museum (Long Island, New York)"
    }
    component "LTA-2R" {
        id: "LTA-2R"
        description: "Apollo 6. Re-entered Earth's atmosphere"
    }
    component "LTA-3A" {
        id: "LTA-3A"
        description: "Not flown. Kansas Cosmosphere and Space Center"
    }
    component "LTA-3DR" {
        id: "LTA-3DR"
        description: "Non-flight descent stage. Franklin Institute"
    }
    component "LTA-5D" {
        id: "LTA-5D"
        description: "Not flown. White Sands Test Facility"
    }
    component "LTA-8A" {
        id: "LTA-8A"
        description: "Thermal-vacuum tests\tGround tests in 1968\tSpace Center Houston"
    }
    component "LTA-10R" {
        id: "LTA-10R"
        description: "Apollo 4. Re-entered Earth's atmosphere"
    }
    component "MSC-16" {
        id: "MSC-16"
        description: "Lunar Module Trainer. Museum of Science and Industry (Chicago)"
    }
    component "TM-5" {
        id: "TM-5"
        description: "Non-flight. Museum of Life and Science (Durham, North Carolina)"
    }
    component "PA-1" {
        id: "PA-1"
        description: "Not flown. White Sands Test Facility"
    }
    component "LM-1" {
        id: "LM-1"
        description: "Apollo 5. Re-entered Earth's atmosphere"
    }
    component "LM-2" {
        id: "LM-2"
        description: "Intended for second uncrewed flight, used instead for ground testing. Landing gear added for drop testing. Lacks Alignment Optical Telescope and flight computer National Air and Space Museum (Washington, D.C.)"
    }
    component "LM-3" {
        id: "LM-3"
        description: "Spider, Apollo 9. Descent and ascent stages reentered Earth's atmosphere separately"
    }
    component "LM-4" {
        id: "LM-4"
        description: "Snoopy, Apollo 10. Descent stage may have hit the Moon, ascent stage in heliocentric orbit. Snoopy is the only flown LM ascent stage known to have survived intact (possibly asteroid 2018 AV2)."
    }
    component "LM-6" {
        id: "LM-6"
        description: "Intrepid, Apollo 12. Descent stage on lunar surface at Ocean of Storms, ascent stage deliberately crashed into Moon"
    }
    component "LM-7" {
        id: "LM-7"
        description: "Aquarius, Apollo 13. Successfully served as a lifeboat for the crew for four days after the critical failure of the Service module. Re-entered Earth's atmosphere and burned up over Fiji"
    }
    component "LM-8" {
        id: "LM-8"
        description: "Antares, Apollo 14. Descent stage on lunar surface at Fra Mauro, ascent stage deliberately crashed into Moon"
    }
    component "LM-9" {
        id: "LM-9"
        description: "Not flown, intended as Apollo 15, last H-class mission. On display at the Kennedy Space Center (Apollo/Saturn V Center)"
    }
    component "LM-10" {
        id: "LM-10"
        description: "Falcon, Apollo 15. Descent stage on lunar surface at Hadley-Apennine, ascent stage deliberately crashed into Moon"
    }
    component "LM-11" {
        id: "LM-11"
        description: "Orion, Apollo 16. Descent stage on lunar surface at Descartes Highlands, ascent stage left in lunar orbit, crashed on Moon"
    }
    component "LM-12" {
        id: "LM-12"
        description: "Challenger, Apollo 17. Descent stage on lunar surface at Taurus-Littrow, ascent stage deliberately crashed into Moon"
    }
    component "LM-13" {
        id: "LM-13"
        description: "Not flown, intended as Apollo 19. individual part defially completed by Grumman, restored and on display at Cradle of Aviation Museum (Long Island, New York). Also used during 1998 miniseries From the Earth to the Moon."
    }
    component "LM-14" {
        id: "LM-14"
        description: "Not flown, intended as Apollo 20.Incomplete, most likely scrapped"
    }
    component "LM-15" {
        id: "LM-15"
        description: "Not flown, intended for modification into Apollo Telescope Mount. Incomplete,scrapped"
    }
    component_exchange "lvToPayload" {
        id: "lvToPayload"
        from_port: "SaturnVInstrumentUnit.payloadInterfacePort"
        to_port: "ApolloSpacecraftLMAdapter.launchVehicleInterfacePort"
    }
    component_exchange "cmSMUmbilical" {
        id: "cmSMUmbilical"
        from_port: "ApolloServiceModule.umbilicalPort"
        to_port: "ApolloCommandModule.umbilicalPort"
    }
    component_exchange "lmInterstage" {
        id: "lmInterstage"
        from_port: "LunarModuleDescentStage.upperStagePort"
        to_port: "LunarModuleAscentStage.lowerStagePort"
    }
    component_exchange "cmLMDocking" {
        id: "cmLMDocking"
        from_port: "ApolloCommandModule.dockingPort"
        to_port: "LunarModuleAscentStage.dockingPort"
    }
    component_exchange "StagingInterface" {
        id: "StagingInterface"
        from_port: "ApolloSpacecraftLMAdapter.csmInterfacePort"
        to_port: "ApolloServiceModule.launchVehicleInterfacePort"
    }
    component_exchange "lesConnection" {
        id: "lesConnection"
        from_port: "ApolloCommandModule.lesInterfacePort"
        to_port: "ApolloLaunchEscapeSystem.cmInterfacePort"
    }
    component_exchange "stage1ToStage2" {
        id: "stage1ToStage2"
        from_port: "S-IC.upperStagePort"
        to_port: "S-II.lowerStagePort"
    }
    component_exchange "stage2ToStage3" {
        id: "stage2ToStage3"
        from_port: "S-II.upperStagePort"
        to_port: "S-IVB.lowerStagePort"
    }
    component_exchange "iuToStage3" {
        id: "iuToStage3"
        from_port: "SaturnVInstrumentUnit.stageControlPort"
        to_port: "S-IVB.controlPort"
    }
    component_exchange "suitToPLSSUmbilical" {
        id: "suitToPLSSUmbilical"
        from_port: "PortableLifeSupportSystem.umbilicalPort"
        to_port: "PressureSuiteAssembly.umbilicalPort"
    }
}

state_machine "apollo11Phases" {
    id: "apollo11Phases"
    owner: "Apollo11Mission"
    state "initial" { id: "initial" }
    state "preparation" { id: "preparation" }
    state "launch" { id: "launch" }
    state "tli" { id: "tli" }
    state "tlc" { id: "tlc" }
    state "loi" { id: "loi" }
    state "lunarOpsDescentPrep" { id: "lunarOpsDescentPrep" }
    state "poweredDescent" { id: "poweredDescent" }
    state "lunarSurfaceOps" { id: "lunarSurfaceOps" }
    state "lunarAscentRendezvous" { id: "lunarAscentRendezvous" }
    state "tei" { id: "tei" }
    state "tec" { id: "tec" }
    state "reentryLanding" { id: "reentryLanding" }
    state "recoveryQuarantine" { id: "recoveryQuarantine" }
    state "final" { id: "final" }
    initial: "initial"
    transition "initial" -> "preparation" {  }
    transition "preparation" -> "launch" { trigger: "PreparationPhaseCompletedNotification" }
    transition "launch" -> "tli" { trigger: "LaunchPhaseCompletedNotification" }
    transition "tli" -> "tlc" { trigger: "TransLunarInjectionPhaseCompletedNotification" }
    transition "tlc" -> "loi" { trigger: "TransLunarCoastPhaseCompletedNotification" }
    transition "loi" -> "lunarOpsDescentPrep" { trigger: "LunarOrbitInsertionPhaseCompletedNotification" }
    transition "lunarOpsDescentPrep" -> "poweredDescent" { trigger: "LunarOrbitOperationsDescentPreparationPhaseCompletedNotification" }
    transition "poweredDescent" -> "lunarSurfaceOps" { trigger: "PoweredDescentPhaseCompletedNotification" }
    transition "lunarSurfaceOps" -> "lunarAscentRendezvous" { trigger: "LunarSurfaceOperationsPhaseCompletedNotification" }
    transition "lunarAscentRendezvous" -> "tei" { trigger: "LunarAscentRendezvousPhaseCompletedNotification" }
    transition "tei" -> "tec" { trigger: "TransEarthInjectionPhaseCompletedNotification" }
    transition "tec" -> "reentryLanding" { trigger: "TransEarthCoastPhaseCompletedNotification" }
    transition "reentryLanding" -> "recoveryQuarantine" { trigger: "ReentryLandingPhaseCompletedNotification" }
    transition "recoveryQuarantine" -> "final" { trigger: "RecoveryQuarantinePhaseCompletedNotification" }
}


trace "load" satisfies "FLR-R001"
trace "load" satisfies "FLR-R002"
trace "board" satisfies "FLR-R003"
trace "board" satisfies "FLR-R004"
trace "countdown" satisfies "FLR-R005"
trace "countdown" satisfies "FLR-R006"
trace "provideThrust1" satisfies "FLR-R007"
trace "provideThrust1" satisfies "FLR-R008"
trace "provideThrust2" satisfies "FLR-R009"
trace "provideThrust2" satisfies "FLR-R010"
trace "provideThrust3" satisfies "FLR-R011"
trace "provideThrust3" satisfies "FLR-R012"
trace "guide" satisfies "FLR-R013"
trace "guide" satisfies "FLR-R014"
trace "observe" satisfies "FLR-R015"
trace "observe" satisfies "FLR-R016"
trace "analyze" satisfies "FLR-R017"
trace "analyze" satisfies "FLR-R018"
trace "burn" satisfies "FLR-R019"
trace "control" satisfies "FLR-R020"
trace "executeTransposition" satisfies "FLR-R021"
trace "executeTransposition" satisfies "FLR-R022"
trace "reconfigure" satisfies "FLR-R023"
trace "burn_" satisfies "FLR-R024"
trace "provideAttitudeControlAndThrust" satisfies "FLR-R025"
trace "trackAndRelay" satisfies "FLR-R026"
trace "calculate" satisfies "FLR-R027"
trace "calculate" satisfies "FLR-R028"
trace "correct" satisfies "FLR-R029"
trace "monitorOnboard" satisfies "FLR-R030"
trace "monitorGround" satisfies "FLR-R031"
trace "comms" satisfies "FLR-R032"
trace "comms" satisfies "FLR-R033"
trace "trackAndRelay" satisfies "FLR-R034"
trace "trackAndRelay" satisfies "FLR-R035"
trace "rest" satisfies "FLR-R036"
trace "rest" satisfies "FLR-R037"
trace "burn_" satisfies "FLR-R038"
trace "control_" satisfies "FLR-R039"
trace "checkout" satisfies "FLR-R040"
trace "checkout" satisfies "FLR-R041"
trace "prep" satisfies "FLR-R042"
trace "prep" satisfies "FLR-R043"
trace "undock" satisfies "FLR-R044"
trace "undock" satisfies "FLR-R045"
trace "burn__" satisfies "FLR-R046"
trace "monitor" satisfies "FLR-R047"
trace "monitor" satisfies "FLR-R048"
trace "pilot" satisfies "FLR-R049"
trace "pilot" satisfies "FLR-R050"
trace "postLand" satisfies "FLR-R051"
trace "postLand" satisfies "FLR-R052"
trace "prepEva" satisfies "FLR-R053"
trace "prepEva" satisfies "FLR-R054"
trace "eva" satisfies "FLR-R055"
trace "eva" satisfies "FLR-R056"
trace "collect" satisfies "FLR-R057"
trace "collect" satisfies "FLR-R058"
trace "deploy" satisfies "FLR-R059"
trace "deploy" satisfies "FLR-R060"
trace "conclude" satisfies "FLR-R061"
trace "conclude" satisfies "FLR-R062"
trace "rest_" satisfies "FLR-R063"
trace "rest_" satisfies "FLR-R064"
trace "ascend" satisfies "FLR-R065"
trace "ascend" satisfies "FLR-R066"
trace "calculate_" satisfies "FLR-R067"
trace "rendezvous" satisfies "FLR-R068"
trace "dock" satisfies "FLR-R069"
trace "dock" satisfies "FLR-R070"
trace "transfer" satisfies "FLR-R071"
trace "transfer" satisfies "FLR-R072"
trace "jettison" satisfies "FLR-R073"
trace "jettison" satisfies "FLR-R074"
trace "burn___" satisfies "FLR-R075"
trace "prep_" satisfies "FLR-R076"
trace "separate" satisfies "FLR-R077"
trace "separate" satisfies "FLR-R078"
trace "guide_" satisfies "FLR-R079"
trace "guide_" satisfies "FLR-R080"
trace "deploy_" satisfies "FLR-R081"
trace "deploy_" satisfies "FLR-R082"
trace "splashdown" satisfies "FLR-R083"
trace "splashdown" satisfies "FLR-R084"
trace "retrieve" satisfies "FLR-R085"
trace "retrieve" satisfies "FLR-R086"
trace "quarantine" satisfies "FLR-R087"
trace "quarantine" satisfies "FLR-R088"
trace "secure" satisfies "FLR-R089"
trace "secure" satisfies "FLR-R090"
trace "trackAndRelay" satisfies "FLR-R091"
trace "pilot" satisfies "FLR-R092"
trace "load" satisfies "FLR-R093"
trace "board" satisfies "FLR-R094"
trace "correct" satisfies "FLR-R095"
trace "provideAttitudeControlAndThrust" satisfies "FLR-R096"
trace "monitor" satisfies "FLR-R097"
trace "provideThrust1" satisfies "FLR-R098"
trace "burn__" satisfies "FLR-R099"
trace "guide_" satisfies "FLR-R100"
trace "burn__" satisfies "FLR-R101"
trace "monitorOnboard" satisfies "FLR-R102"
trace "comms" satisfies "FLR-R103"
trace "eva" satisfies "FLR-R104"
trace "guide" satisfies "FLR-R105"
trace "monitorOnboard" satisfies "FLR-R106"
trace "deploy" satisfies "FLR-R107"
trace "monitorOnboard" satisfies "FLR-R108"
trace "eva" satisfies "FLR-R109"
trace "burn_" satisfies "FLR-R110"
trace "rest" satisfies "FLR-R111"
trace "retrieve" satisfies "FLR-R112"
trace "load" refines "LoadConsumablesAndPropellants"
trace "board" refines "TransferCrewToVehicle"
trace "countdown" refines "PerformPreLaunchCountdown"
trace "provideThrust1" refines "ExecuteLaunchSequence"
trace "provideThrust2" refines "ExecuteLaunchSequence"
trace "provideThrust3" refines "ExecuteLaunchSequence"
trace "guide" refines "MonitorAscentTrajectory"
trace "observe" refines "MonitorAscentTrajectory"
trace "analyze" refines "MonitorAscentTrajectory"
trace "burn" refines "ExecuteTLIBurn"
trace "control" refines "ExecuteTLIBurn"
trace "executeTransposition" refines "PerformTranspositionDockingExtraction"
trace "provideAttitudeControlAndThrust" refines "PerformTranspositionDockingExtraction"
trace "reconfigure" refines "PerformTranspositionDockingExtraction"
trace "calculate" refines "ConductMidCourseCorrections"
trace "correct" refines "ConductMidCourseCorrections"
trace "monitorOnboard" refines "MonitorSpacecraftSystems"
trace "monitorGround" refines "MonitorSpacecraftSystems"
trace "comms" refines "CommunicateWithMissionControl"
trace "trackAndRelay" refines "CommunicateWithMissionControl"
trace "rest" refines "ManageCrewRestPeriods"
trace "burn_" refines "ExecuteLOIBurn"
trace "control_" refines "ExecuteLOIBurn"
trace "checkout" refines "ConductLunarOrbitCheckouts"
trace "prep" refines "PrepareLMforUndocking"
trace "undock" refines "PerformLMUndocking"
trace "burn__" refines "ExecutePoweredDescentBurn"
trace "monitor" refines "MonitorDescentTelemetry"
trace "pilot" refines "PerformManualPiloting"
trace "postLand" refines "ConductLMPostLandingChecks"
trace "prepEva" refines "PrepareForEVA"
trace "provideIndividualLifesupport_" refines "PerformLunarEVA"
trace "eva" refines "PerformLunarEVA"
trace "collect" refines "CollectLunarSamples"
trace "deploy" refines "DeployScientificInstruments"
trace "conclude" refines "ConcludeEVA"
trace "rest_" refines "ManageLunarSurfaceRestPeriod"
trace "ascend" refines "ExecuteLunarAscentBurn"
trace "calculate_" refines "PerformRendezvousManeuvers"
trace "rendezvous" refines "PerformRendezvousManeuvers"
trace "dock" refines "ConductLMCSMDocking"
trace "transfer" refines "TransferCrewAndSamples"
trace "jettison" refines "JettisonLM"
trace "burn___" refines "ExecuteTEIBurn"
trace "prep_" refines "PrepareCMforReentry"
trace "separate" refines "SeparateModules"
trace "guide_" refines "GuideReentryTrajectory"
trace "deploy_" refines "DeployParachutes"
trace "splashdown" refines "ConductSplashdownOperations"
trace "retrieve" refines "RetrieveCrewAndCM"
trace "quarantine" refines "InitiateQuarantineProcedures"
trace "secure" refines "SecureLunarSamples"
trace "ProgrammaticAndPublicAffairsManagement" refines "SHN-N003"
trace "ProgrammaticAndPublicAffairsManagement" refines "SHN-N017"
trace "ProgrammaticAndPublicAffairsManagement" refines "SHN-N018"
trace "ProgrammaticAndPublicAffairsManagement" refines "SHN-N020"
trace "HeavyLiftLaunch" refines "SHN-N001"
trace "HeavyLiftLaunch" refines "SHN-N019"
trace "HeavyLiftLaunch" refines "SHN-N022"
trace "MultiStagePropulsion" refines "SHN-N001"
trace "PrecisionOrbitalMechanics" refines "SHN-N001"
trace "PrecisionOrbitalMechanics" refines "SHN-N022"
trace "AtmosphericReentryAndLanding" refines "SHN-N002"
trace "AtmosphericReentryAndLanding" refines "SHN-N007"
trace "DeepSpaceHabitationAndLifeSupport" refines "SHN-N002"
trace "DeepSpaceHabitationAndLifeSupport" refines "SHN-N007"
trace "DeepSpaceHabitationAndLifeSupport" refines "SHN-N011"
trace "AutonomousAndAssistedNavigationGuidanceAndControl" refines "SHN-N022"
trace "InSpacePropulsion" refines "SHN-N001"
trace "PowerGenerationAndDistribution" refines "SHN-N001"
trace "LunarSurfaceLandingAndAscent" refines "SHN-N004"
trace "LunarSurfaceLandingAndAscent" refines "SHN-N016"
trace "LunarSurfaceLandingAndAscent" refines "SHN-N019"
trace "GlobalTrackingAndCommunication" refines "SHN-N014"
trace "RealtimeMissionControl" refines "SHN-N012"
trace "RealtimeMissionControl" refines "SHN-N014"
trace "RealtimeMissionControl" refines "SHN-N015"
trace "ContingencyPlanningAndRapidResponse" refines "SHN-N002"
trace "ContingencyPlanningAndRapidResponse" refines "SHN-N007"
trace "CrewTrainingAndSimulation" refines "SHN-N008"
trace "CrewTrainingAndSimulation" refines "SHN-N010"
trace "CrewTrainingAndSimulation" refines "SHN-N015"
trace "HumanPerformanceUnderExtremeStress" refines "SHN-N009"
trace "ComplexSoftwareDevelopment" refines "SHN-N001"
trace "ComplexLogisticsAndIntegration" refines "SHN-N001"
trace "LaunchOperationsManagement" refines "SHN-N001"
trace "AdaptabilityAndProblemSolving" refines "SHN-N012"
trace "ExtravehicularActivitySupport" refines "SHN-N011"
trace "LunarMobility" refines "SHN-N004"
trace "ScientificDataAcquisitionAndSampleReturn" refines "SHN-N005"
trace "ScientificDataAcquisitionAndSampleReturn" refines "SHN-N021"
trace "PlanetaryProtection" refines "SHN-N006"
trace "RobustCommunicationAndTelemetry" refines "SHN-N013"
trace "ToolDevelopment" refines "SHN-N008"
trace "RetrieveCrewAndCM" satisfies "HLR-R001"
trace "ExecutePoweredDescentBurn" satisfies "HLR-R002"
trace "ExecuteTLIBurn" satisfies "HLR-R003"
trace "CommunicateWithMissionControl" satisfies "HLR-R004"
trace "CollectLunarSamples" satisfies "HLR-R006"
trace "DeployScientificInstruments" satisfies "HLR-R007"
trace "PerformLunarEVA" satisfies "HLR-R008"
trace "PerformLunarEVA" satisfies "HLR-R009"
trace "InitiateQuarantineProcedures" satisfies "HLR-R010"
trace "MonitorSpacecraftSystems" satisfies "HLR-R026"
trace "ManageCrewRestPeriods" satisfies "HLR-R028"
trace "MonitorSpacecraftSystems" satisfies "HLR-R030"
trace "PerformLunarEVA" satisfies "HLR-R032"
trace "PerformLunarEVA" satisfies "HLR-R033"
trace "PerformLunarEVA" satisfies "HLR-R035"
trace "PerformLunarEVA" satisfies "HLR-R036"
trace "PerformLunarEVA" satisfies "HLR-R037"
trace "CollectLunarSamples" satisfies "HLR-R038"
trace "CollectLunarSamples" satisfies "HLR-R040"
trace "ExecuteLOIBurn" satisfies "HLR-R045"
trace "ExecuteLaunchSequence" satisfies "HLR-R046"
trace "PerformManualPiloting" satisfies "HLR-R047"
trace "CommunicateWithMissionControl" satisfies "HLR-R048"
trace "ExecuteLunarAscentBurn" satisfies "HLR-R049"
trace "CommunicateWithMissionControl" satisfies "HLR-R050"
trace "TransferCrewToVehicle" satisfies "HLR-R052"
trace "ConductLMCSMDocking" satisfies "HLR-R053"
trace "ManageCrewRestPeriods" satisfies "HLR-R054"
trace "ManageCrewRestPeriods" satisfies "HLR-R055"
trace "MonitorSpacecraftSystems" satisfies "HLR-R057"
trace "DeployScientificInstruments" satisfies "HLR-R059"
trace "MonitorSpacecraftSystems" satisfies "HLR-R005"
trace "ConductMidCourseCorrections" satisfies "HLR-R011"
trace "ExecuteLOIBurn" satisfies "HLR-R061"
trace "ExecuteLaunchSequence" satisfies "HLR-R062"
trace "CommunicateWithMissionControl" satisfies "HLR-R063"
trace "DeployScientificInstruments" satisfies "HLR-R064"
trace "MonitorSpacecraftSystems" satisfies "HLR-R065"
trace "CommunicateWithMissionControl" satisfies "HLR-R066"
trace "MonitorSpacecraftSystems" satisfies "HLR-R067"
trace "PerformPreLaunchCountdown" satisfies "HLR-R068"
trace "PerformManualPiloting" satisfies "HLR-R069"
trace "FLR-R001" refines "HLR-R001"
trace "FLR-R002" refines "HLR-R005"
trace "FLR-R003" refines "HLR-R001"
trace "FLR-R004" refines "HLR-R005"
trace "FLR-R005" refines "HLR-R068"
trace "FLR-R006" refines "HLR-R046"
trace "FLR-R007" refines "HLR-R062"
trace "FLR-R008" refines "HLR-R062"
trace "FLR-R009" refines "HLR-R062"
trace "FLR-R010" refines "HLR-R062"
trace "FLR-R011" refines "HLR-R003"
trace "FLR-R012" refines "HLR-R003"
trace "FLR-R013" refines "HLR-R003"
trace "FLR-R014" refines "HLR-R003"
trace "FLR-R015" refines "HLR-R001"
trace "FLR-R016" refines "HLR-R046"
trace "FLR-R017" refines "HLR-R048"
trace "FLR-R018" refines "HLR-R048"
trace "FLR-R019" refines "HLR-R003"
trace "FLR-R020" refines "HLR-R003"
trace "FLR-R021" refines "HLR-R001"
trace "FLR-R022" refines "HLR-R001"
trace "FLR-R023" refines "HLR-R001"
trace "FLR-R024" refines "HLR-R061"
trace "FLR-R025" refines "HLR-R061"
trace "FLR-R026" refines "HLR-R011"
trace "FLR-R027" refines "HLR-R011"
trace "FLR-R028" refines "HLR-R011"
trace "FLR-R029" refines "HLR-R011"
trace "FLR-R030" refines "HLR-R030"
trace "FLR-R031" refines "HLR-R004"
trace "FLR-R032" refines "HLR-R004"
trace "FLR-R033" refines "HLR-R004"
trace "FLR-R034" refines "HLR-R048"
trace "FLR-R035" refines "HLR-R004"
trace "FLR-R036" refines "HLR-R028"
trace "FLR-R037" refines "HLR-R028"
trace "FLR-R038" refines "HLR-R061"
trace "FLR-R039" refines "HLR-R011"
trace "FLR-R040" refines "HLR-R002"
trace "FLR-R041" refines "HLR-R002"
trace "FLR-R042" refines "HLR-R001"
trace "FLR-R043" refines "HLR-R049"
trace "FLR-R044" refines "HLR-R002"
trace "FLR-R045" refines "HLR-R002"
trace "FLR-R046" refines "HLR-R002"
trace "FLR-R047" refines "HLR-R002"
trace "FLR-R048" refines "HLR-R002"
trace "FLR-R049" refines "HLR-R047"
trace "FLR-R050" refines "HLR-R047"
trace "FLR-R051" refines "HLR-R001"
trace "FLR-R052" refines "HLR-R049"
trace "FLR-R053" refines "HLR-R033"
trace "FLR-R054" refines "HLR-R033"
trace "FLR-R055" refines "HLR-R036"
trace "FLR-R056" refines "HLR-R009"
trace "FLR-R057" refines "HLR-R006"
trace "FLR-R058" refines "HLR-R038"
trace "FLR-R059" refines "HLR-R007"
trace "FLR-R060" refines "HLR-R063"
trace "FLR-R061" refines "HLR-R001"
trace "FLR-R062" refines "HLR-R005"
trace "FLR-R063" refines "HLR-R028"
trace "FLR-R064" refines "HLR-R049"
trace "FLR-R065" refines "HLR-R049"
trace "FLR-R066" refines "HLR-R049"
trace "FLR-R067" refines "HLR-R011"
trace "FLR-R068" refines "HLR-R011"
trace "FLR-R069" refines "HLR-R011"
trace "FLR-R070" refines "HLR-R053"
trace "FLR-R071" refines "HLR-R001"
trace "FLR-R072" refines "HLR-R006"
trace "FLR-R073" refines "HLR-R001"
trace "FLR-R074" refines "HLR-R001"
trace "FLR-R075" refines "HLR-R061"
trace "FLR-R076" refines "HLR-R001"
trace "FLR-R077" refines "HLR-R001"
trace "FLR-R078" refines "HLR-R001"
trace "FLR-R079" refines "HLR-R001"
trace "FLR-R080" refines "HLR-R001"
trace "FLR-R081" refines "HLR-R001"
trace "FLR-R082" refines "HLR-R002"
trace "FLR-R083" refines "HLR-R001"
trace "FLR-R084" refines "HLR-R001"
trace "FLR-R085" refines "HLR-R001"
trace "FLR-R086" refines "HLR-R010"
trace "FLR-R087" refines "HLR-R010"
trace "FLR-R088" refines "HLR-R010"
trace "FLR-R089" refines "HLR-R038"
trace "FLR-R090" refines "HLR-R006"
trace "HLR-R011" refines "AutonomousAndAssistedNavigationGuidanceAndControl"
trace "FLR-R092" refines "HumanPerformanceUnderExtremeStress"
trace "FLR-R092" refines "AdaptabilityAndProblemSolving"
trace "FLR-R093" refines "ComplexLogisticsAndIntegration"
trace "FLR-R094" refines "CrewTrainingAndSimulation"
trace "FLR-R095" refines "HLR-R011"
trace "FLR-R096" refines "HLR-R061"
trace "FLR-R097" refines "HLR-R002"
trace "FLR-R098" refines "HLR-R062"
trace "FLR-R099" refines "HLR-R002"
trace "FLR-R100" refines "HLR-R062"
trace "FLR-R101" refines "HLR-R002"
trace "FLR-R102" refines "HLR-R005"
trace "FLR-R103" refines "HLR-R063"
trace "FLR-R104" refines "HLR-R064"
trace "FLR-R105" refines "HLR-R065"
trace "FLR-R106" refines "HLR-R062"
trace "FLR-R107" refines "HLR-R008"
trace "FLR-R108" refines "HLR-R026"
trace "FLR-R109" refines "HLR-R032"
trace "FLR-R110" refines "HLR-R045"
trace "FLR-R111" refines "HLR-R052"
trace "FLR-R112" refines "HLR-R060"
trace "HLR-R001" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R001" refines "AtmosphericReentryAndLanding"
trace "HLR-R001" refines "ContingencyPlanningAndRapidResponse"
trace "HLR-R002" refines "LunarSurfaceLandingAndAscent"
trace "HLR-R002" refines "PrecisionOrbitalMechanics"
trace "HLR-R002" refines "AutonomousAndAssistedNavigationGuidanceAndControl"
trace "HLR-R003" refines "HeavyLiftLaunch"
trace "HLR-R003" refines "MultiStagePropulsion"
trace "HLR-R003" refines "PrecisionOrbitalMechanics"
trace "HLR-R004" refines "GlobalTrackingAndCommunication"
trace "HLR-R004" refines "RobustCommunicationAndTelemetry"
trace "HLR-R005" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R006" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R006" refines "ExtravehicularActivitySupport"
trace "HLR-R007" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R007" refines "ExtravehicularActivitySupport"
trace "HLR-R008" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R009" refines "ExtravehicularActivitySupport"
trace "HLR-R009" refines "LunarMobility"
trace "HLR-R010" refines "PlanetaryProtection"
trace "HLR-R026" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R028" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R030" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R032" refines "RobustCommunicationAndTelemetry"
trace "HLR-R033" refines "ExtravehicularActivitySupport"
trace "HLR-R035" refines "ExtravehicularActivitySupport"
trace "HLR-R035" refines "ToolDevelopment"
trace "HLR-R036" refines "LunarMobility"
trace "HLR-R037" refines "ExtravehicularActivitySupport"
trace "HLR-R038" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R038" refines "PlanetaryProtection"
trace "HLR-R040" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R045" refines "ContingencyPlanningAndRapidResponse"
trace "HLR-R046" refines "ContingencyPlanningAndRapidResponse"
trace "HLR-R046" refines "RealtimeMissionControl"
trace "HLR-R047" refines "HumanPerformanceUnderExtremeStress"
trace "HLR-R048" refines "RealtimeMissionControl"
trace "HLR-R049" refines "LunarSurfaceLandingAndAscent"
trace "HLR-R050" refines "ComplexSoftwareDevelopment"
trace "HLR-R052" refines "CrewTrainingAndSimulation"
trace "HLR-R053" refines "HumanPerformanceUnderExtremeStress"
trace "HLR-R054" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R055" refines "HumanPerformanceUnderExtremeStress"
trace "HLR-R057" refines "DeepSpaceHabitationAndLifeSupport"
trace "HLR-R059" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R060" refines "ComplexLogisticsAndIntegration"
trace "HLR-R061" refines "InSpacePropulsion"
trace "HLR-R062" refines "HeavyLiftLaunch"
trace "HLR-R063" refines "RobustCommunicationAndTelemetry"
trace "HLR-R064" refines "ScientificDataAcquisitionAndSampleReturn"
trace "HLR-R065" refines "ComplexLogisticsAndIntegration"
trace "HLR-R066" refines "ProgrammaticAndPublicAffairsManagement"
trace "HLR-R067" refines "PowerGenerationAndDistribution"
trace "HLR-R068" refines "LaunchOperationsManagement"
trace "HLR-R069" refines "AdaptabilityAndProblemSolving"
trace "CLR-R001" refines "FLR-R008"
trace "CLR-R002" refines "FLR-R001"
trace "CLR-R003" refines "FLR-R013"
trace "CLR-R026" refines "FLR-R014"
trace "CLR-R004" refines "FLR-R010"
trace "CLR-R005" refines "FLR-R001"
trace "CLR-R027" refines "FLR-R010"
trace "CLR-R006" refines "FLR-R011"
trace "CLR-R007" refines "FLR-R019"
trace "CLR-R028" refines "FLR-R020"
trace "CLR-R008" refines "FLR-R013"
trace "CLR-R009" refines "FLR-R014"
trace "CLR-R029" refines "FLR-R017"
trace "CLR-R010" refines "FLR-R016"
trace "CLR-R011" refines "FLR-R016"
trace "CLR-R012" refines "FLR-R004"
trace "CLR-R013" refines "FLR-R080"
trace "CLR-R014" refines "FLR-R081"
trace "CLR-R030" refines "FLR-R023"
trace "CLR-R052" refines "FLR-R028"
trace "CLR-R053" refines "FLR-R096"
trace "CLR-R054" refines "FLR-R097"
trace "CLR-R015" refines "FLR-R024"
trace "CLR-R016" refines "FLR-R025"
trace "CLR-R017" refines "FLR-R030"
trace "CLR-R031" refines "FLR-R031"
trace "CLR-R051" refines "FLR-R102"
trace "CLR-R018" refines "FLR-R046"
trace "CLR-R019" refines "FLR-R002"
trace "CLR-R032" refines "FLR-R065"
trace "CLR-R050" refines "FLR-R082"
trace "CLR-R020" refines "FLR-R066"
trace "CLR-R021" refines "FLR-R062"
trace "CLR-R033" refines "FLR-R067"
trace "CLR-R022" refines "FLR-R053"
trace "CLR-R023" refines "FLR-R055"
trace "CLR-R034" refines "FLR-R055"
trace "CLR-R024" refines "FLR-R054"
trace "CLR-R025" refines "FLR-R053"
trace "CLR-R035" refines "FLR-R053"
trace "CLR-R036" refines "FLR-R008"
trace "CLR-R037" refines "FLR-R014"
trace "CLR-R038" refines "FLR-R013"
trace "CLR-R039" refines "FLR-R017"
trace "CLR-R040" refines "FLR-R030"
trace "CLR-R041" refines "FLR-R025"
trace "CLR-R042" refines "FLR-R023"
trace "CLR-R043" refines "FLR-R071"
trace "CLR-R044" refines "FLR-R065"
trace "CLR-R045" refines "FLR-R053"
trace "CLR-R046" refines "FLR-R008"
trace "CLR-R047" refines "FLR-R022"
trace "CLR-R048" refines "FLR-R016"
trace "CLR-R049" refines "FLR-R016"
trace "CLR-R056" refines "FLR-R024"
trace "CLR-R057" refines "FLR-R099"
trace "CLR-R058" refines "FLR-R066"
trace "CLR-R060" refines "FLR-R100"
trace "CLR-R067" refines "FLR-R103"
trace "CLR-R068" refines "FLR-R104"
trace "CLR-R069" refines "FLR-R059"
trace "CLR-R071" refines "FLR-R105"
trace "CLR-R073" refines "FLR-R106"
trace "CLR-R075" refines "FLR-R107"
trace "CLR-R076" refines "FLR-R108"
trace "CLR-R077" refines "FLR-R109"
trace "CLR-R078" refines "FLR-R110"
trace "CLR-R079" refines "FLR-R111"
trace "CLR-R080" refines "FLR-R112"
trace "CLR-R081" refines "FLR-R003"
trace "CLR-R082" refines "FLR-R006"
trace "CLR-R083" refines "FLR-R014"
trace "CLR-R084" refines "FLR-R004"
trace "CLR-R085" refines "FLR-R005"
trace "CLR-R086" refines "FLR-R007"
trace "CLR-R087" refines "FLR-R009"
trace "CLR-R088" refines "FLR-R012"
trace "CLR-R089" refines "FLR-R015"
trace "CLR-R090" refines "FLR-R018"
trace "CLR-R091" refines "FLR-R019"
trace "CLR-R092" refines "FLR-R021"
trace "CLR-R093" refines "FLR-R022"
trace "CLR-R094" refines "FLR-R026"
trace "CLR-R095" refines "FLR-R027"
trace "CLR-R096" refines "FLR-R033"
trace "CLR-R097" refines "FLR-R034"
trace "CLR-R098" refines "FLR-R035"
trace "CLR-R099" refines "FLR-R036"
trace "CLR-R100" refines "FLR-R037"
trace "CLR-R101" refines "FLR-R039"
trace "CLR-R102" refines "FLR-R042"
trace "CLR-R103" refines "FLR-R043"
trace "CLR-R104" refines "FLR-R044"
trace "CLR-R105" refines "FLR-R045"
trace "CLR-R106" refines "FLR-R047"
trace "CLR-R107" refines "FLR-R048"
trace "CLR-R108" refines "FLR-R051"
trace "CLR-R109" refines "FLR-R052"
trace "CLR-R110" refines "FLR-R056"
trace "CLR-R111" refines "FLR-R057"
trace "CLR-R112" refines "FLR-R059"
trace "CLR-R113" refines "FLR-R061"
trace "CLR-R114" refines "FLR-R064"
trace "CLR-R115" refines "FLR-R066"
trace "CLR-R116" refines "FLR-R068"
trace "CLR-R117" refines "FLR-R079"
trace "S-IC" satisfies "CLR-R001"
trace "S-IC" satisfies "CLR-R002"
trace "S-IC" satisfies "CLR-R003"
trace "S-II" satisfies "CLR-R004"
trace "S-II" satisfies "CLR-R005"
trace "S-IVB" satisfies "CLR-R006"
trace "S-IVB" satisfies "CLR-R007"
trace "SaturnVInstrumentUnit" satisfies "CLR-R008"
trace "SaturnVInstrumentUnit" satisfies "CLR-R009"
trace "ApolloLaunchEscapeSystem" satisfies "CLR-R010"
trace "ApolloLaunchEscapeSystem" satisfies "CLR-R011"
trace "ApolloCommandModule" satisfies "CLR-R012"
trace "ApolloCommandModule" satisfies "CLR-R013"
trace "ApolloCommandModule" satisfies "CLR-R014"
trace "ApolloServiceModule" satisfies "CLR-R015"
trace "ApolloServiceModule" satisfies "CLR-R016"
trace "ApolloServiceModule" satisfies "CLR-R017"
trace "LunarModuleDescentStage" satisfies "CLR-R018"
trace "LunarModuleDescentStage" satisfies "CLR-R019"
trace "LunarModuleDescentStage" satisfies "CLR-R050"
trace "LunarModuleAscentStage" satisfies "CLR-R020"
trace "LunarModuleAscentStage" satisfies "CLR-R021"
trace "PressureSuiteAssembly" satisfies "CLR-R022"
trace "PressureSuiteAssembly" satisfies "CLR-R023"
trace "PortableLifeSupportSystem" satisfies "CLR-R024"
trace "PortableLifeSupportSystem" satisfies "CLR-R025"
trace "S-IC" satisfies "CLR-R026"
trace "S-II" satisfies "CLR-R027"
trace "S-IVB" satisfies "CLR-R028"
trace "SaturnVInstrumentUnit" satisfies "CLR-R029"
trace "ApolloCommandModule" satisfies "CLR-R030"
trace "ApolloServiceModule" satisfies "CLR-R031"
trace "LunarModuleDescentStage" satisfies "CLR-R032"
trace "LunarModuleAscentStage" satisfies "CLR-R033"
trace "PressureSuiteAssembly" satisfies "CLR-R034"
trace "PortableLifeSupportSystem" satisfies "CLR-R035"
trace "stage1ToStage2" satisfies "CLR-R036"
trace "stage2ToStage3" satisfies "CLR-R036"
trace "stage1ToStage2" satisfies "CLR-R037"
trace "stage2ToStage3" satisfies "CLR-R037"
trace "iuToStage3" satisfies "CLR-R038"
trace "iuToStage3" satisfies "CLR-R039"
trace "cmSMUmbilical" satisfies "CLR-R040"
trace "cmSMUmbilical" satisfies "CLR-R041"
trace "cmLMDocking" satisfies "CLR-R042"
trace "cmLMDocking" satisfies "CLR-R043"
trace "lmInterstage" satisfies "CLR-R044"
trace "suitToPLSSUmbilical" satisfies "CLR-R045"
trace "SaturnVInstrumentUnit" satisfies "CLR-R046"
trace "ApolloSpacecraftLMAdapter" satisfies "CLR-R046"
trace "ApolloSpacecraftLMAdapter" satisfies "CLR-R047"
trace "lesConnection" satisfies "CLR-R048"
trace "lesConnection" satisfies "CLR-R049"
trace "S-IC" satisfies "CLR-R086"
trace "S-II" satisfies "CLR-R087"
trace "S-IVB" satisfies "CLR-R088"
trace "S-IVB" satisfies "CLR-R091"
trace "SaturnVInstrumentUnit" satisfies "CLR-R052"
trace "SaturnVInstrumentUnit" satisfies "CLR-R071"
trace "SaturnVInstrumentUnit" satisfies "CLR-R082"
trace "SaturnVInstrumentUnit" satisfies "CLR-R085"
trace "SaturnVInstrumentUnit" satisfies "CLR-R090"
trace "SaturnVInstrumentUnit" satisfies "CLR-R094"
trace "SaturnVInstrumentUnit" satisfies "CLR-R095"
trace "SaturnVInstrumentUnit" satisfies "CLR-R114"
trace "ApolloCommandModule" satisfies "CLR-R051"
trace "ApolloCommandModule" satisfies "CLR-R060"
trace "ApolloCommandModule" satisfies "CLR-R075"
trace "ApolloCommandModule" satisfies "CLR-R076"
trace "ApolloCommandModule" satisfies "CLR-R079"
trace "ApolloCommandModule" satisfies "CLR-R081"
trace "ApolloCommandModule" satisfies "CLR-R084"
trace "ApolloCommandModule" satisfies "CLR-R089"
trace "ApolloCommandModule" satisfies "CLR-R099"
trace "ApolloCommandModule" satisfies "CLR-R100"
trace "ApolloCommandModule" satisfies "CLR-R117"
trace "ApolloServiceModule" satisfies "CLR-R053"
trace "ApolloServiceModule" satisfies "CLR-R056"
trace "ApolloServiceModule" satisfies "CLR-R073"
trace "ApolloServiceModule" satisfies "CLR-R078"
trace "ApolloServiceModule" satisfies "CLR-R093"
trace "ApolloServiceModule" satisfies "CLR-R101"
trace "LunarModuleDescentStage" satisfies "CLR-R054"
trace "LunarModuleDescentStage" satisfies "CLR-R057"
trace "LunarModuleDescentStage" satisfies "CLR-R069"
trace "LunarModuleDescentStage" satisfies "CLR-R107"
trace "LunarModuleDescentStage" satisfies "CLR-R108"
trace "LunarModuleDescentStage" satisfies "CLR-R112"
trace "LunarModuleAscentStage" satisfies "CLR-R058"
trace "LunarModuleAscentStage" satisfies "CLR-R067"
trace "LunarModuleAscentStage" satisfies "CLR-R102"
trace "LunarModuleAscentStage" satisfies "CLR-R103"
trace "LunarModuleAscentStage" satisfies "CLR-R106"
trace "LunarModuleAscentStage" satisfies "CLR-R109"
trace "LunarModuleAscentStage" satisfies "CLR-R115"
trace "LunarModuleAscentStage" satisfies "CLR-R116"
trace "PressureSuiteAssembly" satisfies "CLR-R068"
trace "PressureSuiteAssembly" satisfies "CLR-R110"
trace "PressureSuiteAssembly" satisfies "CLR-R111"
trace "PressureSuiteAssembly" satisfies "CLR-R113"
trace "cmLMDocking" satisfies "CLR-R113"
trace "ApolloSpacecraftLMAdapter" satisfies "CLR-R092"
trace "SaturnVInstrumentUnit" satisfies "CLR-R083"
trace "SaturnVInstrumentUnit" satisfies "CLR-R096"
trace "SaturnVInstrumentUnit" satisfies "CLR-R097"
trace "SaturnVInstrumentUnit" satisfies "CLR-R098"
trace "SaturnVInstrumentUnit" satisfies "CLR-R080"
diagram "stages interconnection" {
    view: lab
    shows: ["stages"]
}

diagram "ApolloProgram interconnection" {
    view: lab
    shows: ["ApolloProgram"]
}

diagram "LaunchSite interconnection" {
    view: lab
    shows: ["LaunchSite"]
}

diagram "SaturnV interconnection" {
    view: lab
    shows: ["SaturnV"]
}

diagram "PerformLunarMission dataflow" {
    view: lab
    shows: ["load", "board", "countdown", "provideThrust1", "provideThrust2", "provideThrust3", "guide", "observe", "analyze", "executeTransposition", "provideAttitudeControlAndThrust", "reconfigure", "burn", "control", "calculate", "correct", "monitorOnboard", "monitorGround", "comms", "trackAndRelay", "rest", "burn_", "control_", "checkout", "prep", "undock", "burn__", "monitor", "pilot", "postLand", "prepEva", "provideIndividualLifesupport_", "eva", "collect", "deploy", "conclude", "rest_", "ascend", "calculate_", "rendezvous", "dock", "transfer", "jettison", "burn___", "monitorSystems", "monitorTelemetry", "transmitAndReceive", "trackAndRelay_", "prep_", "separate", "guide_", "deploy_", "splashdown", "retrieve", "quarantine", "secure"]
}

