if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'pp');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'pp', 'Folder', project_path);
end
cd(project_path);
PhysicalQuantity_1 = Simulink.Signal;
PhysicalQuantity_1.Unit = 'm';
PhysicalQuantity_1.Description = 'ModelerId:ff9ea6c7-6ce1-41b4-9e65-c3cfe738740f';
assignin('base', 'PhysicalQuantity_1', PhysicalQuantity_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
elems(1).Description = 'ModelerId:600749cb-c43c-4ba7-9ac1-c1e910ef08ae';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Description = 'ModelerId:51958f33-18c2-49ed-b782-210331341a85';
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItemElement_1';
elems(1).Description = 'ModelerId:14443d90-d032-4a53-90b5-248cd67306a3';
elems(1).DataType = 'Bus: busClass_Class_1';
busEI_sub_ExchangeItem_1 = Simulink.Bus;
busEI_sub_ExchangeItem_1.Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
busEI_sub_ExchangeItem_1.Elements = elems;
assignin('base', 'busEI_sub_ExchangeItem_1', busEI_sub_ExchangeItem_1);
clear elems;
if isfile('data/PA/PA_Data.sldd')
	PA_Data = Simulink.data.dictionary.open('data/PA/PA_Data.sldd');
else
	PA_Data = Simulink.data.dictionary.create('data/PA/PA_Data.sldd');
end
importFromBaseWorkspace(PA_Data, 'varList', {'PhysicalQuantity_1', 'busClass_Class_1', 'busEI_sub_ExchangeItem_1'}, 'existingVarsAction','overwrite');
saveChanges(PA_Data);
clear PA_Data;
file = proj.addFile('data/PA/PA_Data.sldd');
addLabel(file, 'Capella Layer', 'PA');
clear elems;
busFE_FunctionalExchange_2 = Simulink.Bus;
busFE_FunctionalExchange_2.Description = 'ModelerId:8aaa3039-dc85-46ef-a201-4d26e7873993';
assignin('base', 'busFE_FunctionalExchange_2', busFE_FunctionalExchange_2);
clear elems;
clear elems;
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:be094c00-f0b3-440a-a0c2-1e7febcd20df';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'sub_ExchangeItem_1';
elems(1).Description = 'ModelerId:97d4d41f-e227-46f4-8cff-7316b2a316ef';
elems(1).DataType = 'Bus: busEI_sub_ExchangeItem_1';
busFE_sub_FunctionalExchange_1 = Simulink.Bus;
busFE_sub_FunctionalExchange_1.Description = 'ModelerId:65a7b34a-9a23-47f3-81b7-d22c644c694d';
busFE_sub_FunctionalExchange_1.Elements = elems;
assignin('base', 'busFE_sub_FunctionalExchange_1', busFE_sub_FunctionalExchange_1);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_2', 'busFE_FunctionalExchange_1', 'busFE_sub_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
addDataSource(PA_FunctionalExchanges_Categories, 'PA_Data.sldd');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_sub_1.sldd')
	PA_sub_1 = Simulink.data.dictionary.open('data/PA/PA_sub_1.sldd');
else
	PA_sub_1 = Simulink.data.dictionary.create('data/PA/PA_sub_1.sldd');
end
addDataSource(PA_sub_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_sub_1);
clear PA_sub_1;
file = proj.addFile('data/PA/PA_sub_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_2.sldd')
	PA_leaf_2 = Simulink.data.dictionary.open('data/PA/PA_leaf_2.sldd');
else
	PA_leaf_2 = Simulink.data.dictionary.create('data/PA/PA_leaf_2.sldd');
end
addDataSource(PA_leaf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_2);
clear PA_leaf_2;
file = proj.addFile('data/PA/PA_leaf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf_1.sldd')
	PA_leaf_1 = Simulink.data.dictionary.open('data/PA/PA_leaf_1.sldd');
else
	PA_leaf_1 = Simulink.data.dictionary.create('data/PA/PA_leaf_1.sldd');
end
addDataSource(PA_leaf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf_1);
clear PA_leaf_1;
file = proj.addFile('data/PA/PA_leaf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom_1.sldd')
	PA_bottom_1 = Simulink.data.dictionary.open('data/PA/PA_bottom_1.sldd');
else
	PA_bottom_1 = Simulink.data.dictionary.create('data/PA/PA_bottom_1.sldd');
end
addDataSource(PA_bottom_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom_1);
clear PA_bottom_1;
file = proj.addFile('data/PA/PA_bottom_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_container_1.sldd')
	PA_container_1 = Simulink.data.dictionary.open('data/PA/PA_container_1.sldd');
else
	PA_container_1 = Simulink.data.dictionary.create('data/PA/PA_container_1.sldd');
end
addDataSource(PA_container_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_container_1);
clear PA_container_1;
file = proj.addFile('data/PA/PA_container_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'PhysicalQuantity_1' 'busClass_Class_1' 'busEI_sub_ExchangeItem_1' 'busFE_FunctionalExchange_1' 'busFE_sub_FunctionalExchange_1' 'busFE_FunctionalExchange_2'");
new_system('PA_bottom_1');
set_param('PA_bottom_1', 'Tag', 'ModelerId:18d24771-6019-4037-bbd9-8be358085c80');
set_param('PA_bottom_1', 'DataDictionary', 'PA_bottom_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom_1/FunctionalExchange_2');
set_param('PA_bottom_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_bottom_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_bottom_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_bottom_1/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom_1/sub_FunctionalExchange_1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_bottom_1/sub_FunctionalExchange_1', 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom_1/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom_1/sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom_1');
catch
end
save_system('PA_bottom_1', fullfile(project_path, 'models/PA/PA_bottom_1.slx'));
file = proj.addFile('models/PA/PA_bottom_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_sub_1');
set_param('PA_sub_1', 'Tag', 'ModelerId:003b2df1-c09c-48e7-9811-5b9563057a96');
set_param('PA_sub_1', 'DataDictionary', 'PA_sub_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_sub_1/FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_sub_1/FunctionalExchange_1', 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
add_block('simulink/Ports & Subsystems/Out1', 'PA_sub_1/sub_FunctionalExchange_1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_sub_1/sub_FunctionalExchange_1', 'Tag', 'ModelerId:b4ad6c0a-7966-420f-bb2f-696a36b99c08');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_sub_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_sub_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_sub_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_sub_1/sub_FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_sub_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_sub_1');
catch
end
save_system('PA_sub_1', fullfile(project_path, 'models/PA/PA_sub_1.slx'));
file = proj.addFile('models/PA/PA_sub_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_container_1');
set_param('PA_container_1', 'Tag', 'ModelerId:9df477a4-df2e-45ca-b5b7-e3d8beb58477');
set_param('PA_container_1', 'DataDictionary', 'PA_container_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_container_1/FunctionalExchange_1');
set_param('PA_container_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_container_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_container_1/FunctionalExchange_1', 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
add_block('simulink/Ports & Subsystems/Out1', 'PA_container_1/FunctionalExchange_2');
set_param('PA_container_1/FunctionalExchange_2', 'Port', '1');
set_param('PA_container_1/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_container_1/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_container_1/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_container_1/Reference_to_sub_1');
set_param('PA_container_1/Reference_to_sub_1', 'Tag', 'ModelerId:003b2df1-c09c-48e7-9811-5b9563057a96');
set_param('PA_container_1/Reference_to_sub_1', 'ModelFile', 'PA_sub_1')
handles = get_param('PA_container_1/Reference_to_sub_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:b4ad6c0a-7966-420f-bb2f-696a36b99c08');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_container_1/sub_2');
set_param('PA_container_1/sub_2', 'Tag', 'ModelerId:2b467853-ab2c-436a-8cf3-fb59304ae4c0');
CapSLBridge.internal.cleanSubsystem('PA_container_1/sub_2');
add_block('simulink/Ports & Subsystems/Out1', 'PA_container_1/sub_2/FunctionalExchange_2');
set_param('PA_container_1/sub_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_container_1/sub_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_container_1/sub_2/FunctionalExchange_2', 'BusOutputAsStruct', 'on');
set_param('PA_container_1/sub_2/FunctionalExchange_2', 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
add_block('simulink/Ports & Subsystems/In1', 'PA_container_1/sub_2/sub_FunctionalExchange_1');
set_param('PA_container_1/sub_2/sub_FunctionalExchange_1', 'Port', '1');
set_param('PA_container_1/sub_2/sub_FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_sub_FunctionalExchange_1');
set_param('PA_container_1/sub_2/sub_FunctionalExchange_1', 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_container_1/sub_2/Reference_to_bottom_1');
set_param('PA_container_1/sub_2/Reference_to_bottom_1', 'Tag', 'ModelerId:18d24771-6019-4037-bbd9-8be358085c80');
set_param('PA_container_1/sub_2/Reference_to_bottom_1', 'ModelFile', 'PA_bottom_1')
handles = get_param('PA_container_1/sub_2/Reference_to_bottom_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear handles;
handles = get_param('PA_container_1/sub_2/Reference_to_bottom_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container_1/sub_2/FunctionalExchange_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container_1/sub_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear line handles source target;
handles = get_param('PA_container_1/sub_2/sub_FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container_1/sub_2/Reference_to_bottom_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container_1/sub_2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:92dfd0ca-0b71-4235-b595-9c326ec57e82');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_container_1/sub_2');
catch
end
handles = get_param('PA_container_1/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container_1/Reference_to_sub_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:260b2635-00bc-43f8-a1b1-12f639e39480');
clear line handles source target;
handles = get_param('PA_container_1/sub_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container_1/FunctionalExchange_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_2');
set_param( line, 'Tag', 'ModelerId:3368f681-65e9-47be-990c-c9a61bb23858');
clear line handles source target;
handles = get_param('PA_container_1/Reference_to_sub_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container_1/sub_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container_1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub_FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:65a7b34a-9a23-47f3-81b7-d22c644c694d');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_container_1');
catch
end
save_system('PA_container_1', fullfile(project_path, 'models/PA/PA_container_1.slx'));
file = proj.addFile('models/PA/PA_container_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_1');
set_param('PA_leaf_1', 'Tag', 'ModelerId:2df54e0f-10fc-4498-a73d-5ef1a75ee70a');
set_param('PA_leaf_1', 'DataDictionary', 'PA_leaf_1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf_1/FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf_1/FunctionalExchange_1', 'Tag', 'ModelerId:29dd7f04-b10f-41f0-8bd5-cb04a7d20f17');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_1');
catch
end
save_system('PA_leaf_1', fullfile(project_path, 'models/PA/PA_leaf_1.slx'));
file = proj.addFile('models/PA/PA_leaf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf_2');
set_param('PA_leaf_2', 'Tag', 'ModelerId:178afa0b-0f08-41d8-ac7f-9db851bdf4bf');
set_param('PA_leaf_2', 'DataDictionary', 'PA_leaf_2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf_2/FunctionalExchange_2');
set_param('PA_leaf_2/FunctionalExchange_2', 'Port', '1');
set_param('PA_leaf_2/FunctionalExchange_2', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_2');
set_param('PA_leaf_2/FunctionalExchange_2', 'Tag', 'ModelerId:905fbdf8-70c6-407e-b6af-646c6a0f4a33');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf_2/FunctionalExchange_2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf_2');
catch
end
save_system('PA_leaf_2', fullfile(project_path, 'models/PA/PA_leaf_2.slx'));
file = proj.addFile('models/PA/PA_leaf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
