if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject21');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject21', 'Folder', project_path);
end
cd(project_path);
clear elems;
busEI_ExchangeItem_1 = Simulink.Bus;
busEI_ExchangeItem_1.Description = 'ModelerId:4a23b832-e92d-4a08-9f21-2832cc6c09c7';
assignin('base', 'busEI_ExchangeItem_1', busEI_ExchangeItem_1);
clear elems;
clear elems;
busEI_ExchangeItem_2 = Simulink.Bus;
busEI_ExchangeItem_2.Description = 'ModelerId:8c7ea9fd-18e6-4efc-a156-767a14af1cf9';
assignin('base', 'busEI_ExchangeItem_2', busEI_ExchangeItem_2);
clear elems;
clear elems;
busEI_ExchangeItem_3 = Simulink.Bus;
busEI_ExchangeItem_3.Description = 'ModelerId:942726ec-5d12-45db-a7e7-f74ee6c99ac5';
assignin('base', 'busEI_ExchangeItem_3', busEI_ExchangeItem_3);
clear elems;
if isfile('data/LA/LA_Interfaces.sldd')
	LA_Interfaces = Simulink.data.dictionary.open('data/LA/LA_Interfaces.sldd');
else
	LA_Interfaces = Simulink.data.dictionary.create('data/LA/LA_Interfaces.sldd');
end
importFromBaseWorkspace(LA_Interfaces, 'varList', {'busEI_ExchangeItem_1', 'busEI_ExchangeItem_2', 'busEI_ExchangeItem_3'}, 'existingVarsAction','overwrite');
saveChanges(LA_Interfaces);
clear LA_Interfaces;
file = proj.addFile('data/LA/LA_Interfaces.sldd');
addLabel(file, 'Capella Layer', 'LA');
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_1';
elems(1).Description = 'ModelerId:4a23b832-e92d-4a08-9f21-2832cc6c09c7';
elems(1).DataType = 'Bus: busEI_ExchangeItem_1';
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:42bdba62-fd25-4d46-a137-6664b4eea726';
busFE_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_2';
elems(1).Description = 'ModelerId:8c7ea9fd-18e6-4efc-a156-767a14af1cf9';
elems(1).DataType = 'Bus: busEI_ExchangeItem_2';
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:fbad045d-baef-4f0c-950a-81270c3f68b0';
busFE_FunctionalExchange_2.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'ExchangeItem_3';
elems(1).Description = 'ModelerId:942726ec-5d12-45db-a7e7-f74ee6c99ac5';
elems(1).DataType = 'Bus: busEI_ExchangeItem_3';
busFE_FunctionalExchange_3 = Simulink.Bus;
busFE_FunctionalExchange_3.Description = 'ModelerId:746b60a9-5901-48ed-a0c1-63ea2c168d6e';
busFE_FunctionalExchange_3.Elements = elems;
assignin('base', 'busFE_FunctionalExchange_3', busFE_FunctionalExchange_3);
clear elems;
if isfile('data/LA/LA_Logical_Architecture.sldd')
	LA_Logical_Architecture = Simulink.data.dictionary.open('data/LA/LA_Logical_Architecture.sldd');
else
	LA_Logical_Architecture = Simulink.data.dictionary.create('data/LA/LA_Logical_Architecture.sldd');
end
importFromBaseWorkspace(LA_Logical_Architecture, 'varList', {'busFE_FunctionalExchange_1', 'busFE_FunctionalExchange_2', 'busFE_FunctionalExchange_3'}, 'existingVarsAction','overwrite');
saveChanges(LA_Logical_Architecture);
clear LA_Logical_Architecture;
file = proj.addFile('data/LA/LA_Logical_Architecture.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_1.sldd')
	LA_LogicalFunction_1 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_1.sldd');
else
	LA_LogicalFunction_1 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_1.sldd');
end
addDataSource(LA_LogicalFunction_1, 'LA_Logical_Architecture.sldd');
saveChanges(LA_LogicalFunction_1);
clear LA_LogicalFunction_1;
file = proj.addFile('data/LA/LA_LogicalFunction_1.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_2.sldd')
	LA_LogicalFunction_2 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_2.sldd');
else
	LA_LogicalFunction_2 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_2.sldd');
end
addDataSource(LA_LogicalFunction_2, 'LA_Logical_Architecture.sldd');
saveChanges(LA_LogicalFunction_2);
clear LA_LogicalFunction_2;
file = proj.addFile('data/LA/LA_LogicalFunction_2.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_3.sldd')
	LA_LogicalFunction_3 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_3.sldd');
else
	LA_LogicalFunction_3 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_3.sldd');
end
addDataSource(LA_LogicalFunction_3, 'LA_Logical_Architecture.sldd');
saveChanges(LA_LogicalFunction_3);
clear LA_LogicalFunction_3;
file = proj.addFile('data/LA/LA_LogicalFunction_3.sldd');
addLabel(file, 'Capella Layer', 'LA');
if isfile('data/LA/LA_LogicalFunction_4.sldd')
	LA_LogicalFunction_4 = Simulink.data.dictionary.open('data/LA/LA_LogicalFunction_4.sldd');
else
	LA_LogicalFunction_4 = Simulink.data.dictionary.create('data/LA/LA_LogicalFunction_4.sldd');
end
addDataSource(LA_LogicalFunction_4, 'LA_Logical_Architecture.sldd');
saveChanges(LA_LogicalFunction_4);
clear LA_LogicalFunction_4;
file = proj.addFile('data/LA/LA_LogicalFunction_4.sldd');
addLabel(file, 'Capella Layer', 'LA');
evalin('base', "clear 'busFE_FunctionalExchange_1' 'busEI_ExchangeItem_3' 'busFE_FunctionalExchange_3' 'busFE_FunctionalExchange_2' 'busEI_ExchangeItem_1' 'busEI_ExchangeItem_2'");
new_system('LA_LogicalFunction_4');
set_param('LA_LogicalFunction_4', 'Tag', 'ModelerId:1284dab8-5ef9-4400-8677-bffb22fd0cf4');
set_param('LA_LogicalFunction_4', 'DataDictionary', 'LA_LogicalFunction_4.sldd');
add_block('simulink/Ports & Subsystems/In1', 'LA_LogicalFunction_4/FunctionalExchange_3');
set_param('LA_LogicalFunction_4/FunctionalExchange_3', 'Port', '1');
set_param('LA_LogicalFunction_4/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('LA_LogicalFunction_4/FunctionalExchange_3', 'Tag', 'ModelerId:ec70c17a-713d-4a22-b40a-7531aa565a5a');
t_Block = add_block('simulink/Sinks/Terminator', 'LA_LogicalFunction_4/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_4/FunctionalExchange_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_LogicalFunction_4', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_4');
catch
end
save_system('LA_LogicalFunction_4', fullfile(project_path, 'models/LA/LA_LogicalFunction_4.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_4.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_LogicalFunction_2');
set_param('LA_LogicalFunction_2', 'Tag', 'ModelerId:dbf31a91-9c17-4e7e-ae2c-536f969eb3bf');
set_param('LA_LogicalFunction_2', 'DataDictionary', 'LA_LogicalFunction_2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'LA_LogicalFunction_2/FunctionalExchange_2');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'Port', '1');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('LA_LogicalFunction_2/FunctionalExchange_2', 'Tag', 'ModelerId:21270ffc-fb05-4cb5-8509-8596bc632192');
add_block('simulink/Ports & Subsystems/In1', 'LA_LogicalFunction_2/FunctionalExchange_1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'Port', '1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('LA_LogicalFunction_2/FunctionalExchange_1', 'Tag', 'ModelerId:89205ca5-e540-49d2-883c-6e54473dc310');
add_block('simulink/Ports & Subsystems/Out1', 'LA_LogicalFunction_2/FunctionalExchange_3');
set_param('LA_LogicalFunction_2/FunctionalExchange_3', 'Port', '2');
set_param('LA_LogicalFunction_2/FunctionalExchange_3', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_3');
set_param('LA_LogicalFunction_2/FunctionalExchange_3', 'BusOutputAsStruct', 'on');
set_param('LA_LogicalFunction_2/FunctionalExchange_3', 'Tag', 'ModelerId:b2505bc0-9af4-4bd7-8b3d-ed62b0aed764');
g_Block = add_block('simulink/Sources/Ground', 'LA_LogicalFunction_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_LogicalFunction_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
g_Block = add_block('simulink/Sources/Ground', 'LA_LogicalFunction_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_2/FunctionalExchange_3', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_LogicalFunction_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'LA_LogicalFunction_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_LogicalFunction_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_2');
catch
end
save_system('LA_LogicalFunction_2', fullfile(project_path, 'models/LA/LA_LogicalFunction_2.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_2.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_LogicalFunction_1');
set_param('LA_LogicalFunction_1', 'Tag', 'ModelerId:4c8677e6-80fa-44ba-bedf-d9625d79e36f');
set_param('LA_LogicalFunction_1', 'DataDictionary', 'LA_LogicalFunction_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'LA_LogicalFunction_1/FunctionalExchange_1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'Port', '1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('LA_LogicalFunction_1/FunctionalExchange_1', 'Tag', 'ModelerId:18b45208-beb6-4990-a0dc-cf22fd15a9f6');
g_Block = add_block('simulink/Sources/Ground', 'LA_LogicalFunction_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_LogicalFunction_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_1');
catch
end
save_system('LA_LogicalFunction_1', fullfile(project_path, 'models/LA/LA_LogicalFunction_1.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_1.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_FunctionalChain_2');
set_param('LA_FunctionalChain_2', 'Tag', 'ModelerId:14f11d7a-1d1f-4d22-90e1-29dce96ad5d3');
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_2/Reference_to_LogicalFunction_1');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_1', 'Tag', 'ModelerId:4c8677e6-80fa-44ba-bedf-d9625d79e36f');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_1', 'ModelFile', 'LA_LogicalFunction_1')
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:18b45208-beb6-4990-a0dc-cf22fd15a9f6');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_2/Reference_to_LogicalFunction_2');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'Tag', 'ModelerId:dbf31a91-9c17-4e7e-ae2c-536f969eb3bf');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'ModelFile', 'LA_LogicalFunction_2')
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:21270ffc-fb05-4cb5-8509-8596bc632192');
clear handles;
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'porthandles');
set_param(handles.Outport(2), 'Tag', 'ModelerId:b2505bc0-9af4-4bd7-8b3d-ed62b0aed764');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_2/Reference_to_LogicalFunction_4');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_4', 'Tag', 'ModelerId:1284dab8-5ef9-4400-8677-bffb22fd0cf4');
set_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_4', 'ModelFile', 'LA_LogicalFunction_4')
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:42bdba62-fd25-4d46-a137-6664b4eea726');
clear line handles source target;
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_2', 'PortHandles');
source = handles.Outport(2);
handles = get_param('LA_FunctionalChain_2/Reference_to_LogicalFunction_4', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_3');
set_param( line, 'Tag', 'ModelerId:746b60a9-5901-48ed-a0c1-63ea2c168d6e');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LA_FunctionalChain_2');
catch
end
save_system('LA_FunctionalChain_2', fullfile(project_path, 'models/LA/LA_FunctionalChain_2.slx'));
file = proj.addFile('models/LA/LA_FunctionalChain_2.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_LogicalFunction_3');
set_param('LA_LogicalFunction_3', 'Tag', 'ModelerId:451fa570-7abe-4e3f-9d00-3a1076db5c26');
set_param('LA_LogicalFunction_3', 'DataDictionary', 'LA_LogicalFunction_3.sldd');
add_block('simulink/Ports & Subsystems/In1', 'LA_LogicalFunction_3/FunctionalExchange_2');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'Port', '1');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('LA_LogicalFunction_3/FunctionalExchange_2', 'Tag', 'ModelerId:0514c022-2422-431a-977b-2e12dc809a27');
t_Block = add_block('simulink/Sinks/Terminator', 'LA_LogicalFunction_3/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_LogicalFunction_3/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_LogicalFunction_3', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_LogicalFunction_3');
catch
end
save_system('LA_LogicalFunction_3', fullfile(project_path, 'models/LA/LA_LogicalFunction_3.slx'));
file = proj.addFile('models/LA/LA_LogicalFunction_3.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_FunctionalChain_1');
set_param('LA_FunctionalChain_1', 'Tag', 'ModelerId:0fb11410-6986-4f32-ae12-44129e3bfc8f');
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_1');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'Tag', 'ModelerId:4c8677e6-80fa-44ba-bedf-d9625d79e36f');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'ModelFile', 'LA_LogicalFunction_1')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:18b45208-beb6-4990-a0dc-cf22fd15a9f6');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_2');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'Tag', 'ModelerId:dbf31a91-9c17-4e7e-ae2c-536f969eb3bf');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'ModelFile', 'LA_LogicalFunction_2')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:21270ffc-fb05-4cb5-8509-8596bc632192');
clear handles;
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'porthandles');
set_param(handles.Outport(2), 'Tag', 'ModelerId:b2505bc0-9af4-4bd7-8b3d-ed62b0aed764');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA_FunctionalChain_1/Reference_to_LogicalFunction_3');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'Tag', 'ModelerId:451fa570-7abe-4e3f-9d00-3a1076db5c26');
set_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'ModelFile', 'LA_LogicalFunction_3')
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_3', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:fbad045d-baef-4f0c-950a-81270c3f68b0');
clear line handles source target;
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA_FunctionalChain_1/Reference_to_LogicalFunction_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA_FunctionalChain_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:42bdba62-fd25-4d46-a137-6664b4eea726');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LA_FunctionalChain_1');
catch
end
save_system('LA_FunctionalChain_1', fullfile(project_path, 'models/LA/LA_FunctionalChain_1.slx'));
file = proj.addFile('models/LA/LA_FunctionalChain_1.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
