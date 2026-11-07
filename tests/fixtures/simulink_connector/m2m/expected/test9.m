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
clear elems;
busFE_leaf1_to_bottom1 = Simulink.Bus;
busFE_leaf1_to_bottom1.Description = 'ModelerId:f29a19c0-7886-4b9b-9ff0-df0bb36dbcb8';
assignin('base', 'busFE_leaf1_to_bottom1', busFE_leaf1_to_bottom1);
clear elems;
clear elems;
busFE_bottom2_to_leaf2 = Simulink.Bus;
busFE_bottom2_to_leaf2.Description = 'ModelerId:48620e46-8811-4368-99a4-4d6401053a1e';
assignin('base', 'busFE_bottom2_to_leaf2', busFE_bottom2_to_leaf2);
clear elems;
clear elems;
busFE_sub = Simulink.Bus;
busFE_sub.Description = 'ModelerId:a06d23f6-6296-4637-93b3-1780ead25ffc';
assignin('base', 'busFE_sub', busFE_sub);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_leaf1_to_bottom1', 'busFE_bottom2_to_leaf2', 'busFE_sub'}, 'existingVarsAction','overwrite');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf1.sldd')
	PA_leaf1 = Simulink.data.dictionary.open('data/PA/PA_leaf1.sldd');
else
	PA_leaf1 = Simulink.data.dictionary.create('data/PA/PA_leaf1.sldd');
end
addDataSource(PA_leaf1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf1);
clear PA_leaf1;
file = proj.addFile('data/PA/PA_leaf1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_leaf2.sldd')
	PA_leaf2 = Simulink.data.dictionary.open('data/PA/PA_leaf2.sldd');
else
	PA_leaf2 = Simulink.data.dictionary.create('data/PA/PA_leaf2.sldd');
end
addDataSource(PA_leaf2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf2);
clear PA_leaf2;
file = proj.addFile('data/PA/PA_leaf2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom2.sldd')
	PA_bottom2 = Simulink.data.dictionary.open('data/PA/PA_bottom2.sldd');
else
	PA_bottom2 = Simulink.data.dictionary.create('data/PA/PA_bottom2.sldd');
end
addDataSource(PA_bottom2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom2);
clear PA_bottom2;
file = proj.addFile('data/PA/PA_bottom2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_bottom1.sldd')
	PA_bottom1 = Simulink.data.dictionary.open('data/PA/PA_bottom1.sldd');
else
	PA_bottom1 = Simulink.data.dictionary.create('data/PA/PA_bottom1.sldd');
end
addDataSource(PA_bottom1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_bottom1);
clear PA_bottom1;
file = proj.addFile('data/PA/PA_bottom1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Top_3.sldd')
	PA_Top_3 = Simulink.data.dictionary.open('data/PA/PA_Top_3.sldd');
else
	PA_Top_3 = Simulink.data.dictionary.create('data/PA/PA_Top_3.sldd');
end
addDataSource(PA_Top_3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Top_3);
clear PA_Top_3;
file = proj.addFile('data/PA/PA_Top_3.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'busFE_bottom2_to_leaf2' 'busFE_leaf1_to_bottom1' 'busFE_sub'");
new_system('PA_bottom1');
set_param('PA_bottom1', 'Tag', 'ModelerId:0d18653e-f4cb-4b57-9d99-6fa587a4634f');
set_param('PA_bottom1', 'DataDictionary', 'PA_bottom1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom1/leaf1_to_bottom1');
set_param('PA_bottom1/leaf1_to_bottom1', 'Port', '1');
set_param('PA_bottom1/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_bottom1/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom1/sub');
set_param('PA_bottom1/sub', 'Port', '1');
set_param('PA_bottom1/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_bottom1/sub', 'BusOutputAsStruct', 'on');
set_param('PA_bottom1/sub', 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom1/leaf1_to_bottom1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom1/sub', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom1');
catch
end
save_system('PA_bottom1', fullfile(project_path, 'models/PA/PA_bottom1.slx'));
file = proj.addFile('models/PA/PA_bottom1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_bottom2');
set_param('PA_bottom2', 'Tag', 'ModelerId:ffa504fd-8e29-4095-8e15-dd41f7e08583');
set_param('PA_bottom2', 'DataDictionary', 'PA_bottom2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_bottom2/bottom2_to_leaf2');
set_param('PA_bottom2/bottom2_to_leaf2', 'Port', '1');
set_param('PA_bottom2/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_bottom2/bottom2_to_leaf2', 'BusOutputAsStruct', 'on');
set_param('PA_bottom2/bottom2_to_leaf2', 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
add_block('simulink/Ports & Subsystems/In1', 'PA_bottom2/sub');
set_param('PA_bottom2/sub', 'Port', '1');
set_param('PA_bottom2/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_bottom2/sub', 'Tag', 'ModelerId:0bc182b2-abb3-4b9d-93f4-ccc61967c15d');
g_Block = add_block('simulink/Sources/Ground', 'PA_bottom2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_bottom2/bottom2_to_leaf2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_bottom2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
t_Block = add_block('simulink/Sinks/Terminator', 'PA_bottom2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_bottom2/sub', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_bottom2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_bottom2');
catch
end
save_system('PA_bottom2', fullfile(project_path, 'models/PA/PA_bottom2.slx'));
file = proj.addFile('models/PA/PA_bottom2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf2');
set_param('PA_leaf2', 'Tag', 'ModelerId:ee115b68-09f3-482f-9584-282bab1f6864');
set_param('PA_leaf2', 'DataDictionary', 'PA_leaf2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf2/bottom2_to_leaf2');
set_param('PA_leaf2/bottom2_to_leaf2', 'Port', '1');
set_param('PA_leaf2/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_leaf2/bottom2_to_leaf2', 'Tag', 'ModelerId:16b7bebe-fe34-4f96-b62c-2968eb28dd4b');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf2/bottom2_to_leaf2', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf2');
catch
end
save_system('PA_leaf2', fullfile(project_path, 'models/PA/PA_leaf2.slx'));
file = proj.addFile('models/PA/PA_leaf2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf1');
set_param('PA_leaf1', 'Tag', 'ModelerId:1e5ed3de-ecc9-4902-ba70-71eeb58c021b');
set_param('PA_leaf1', 'DataDictionary', 'PA_leaf1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf1/leaf1_to_bottom1');
set_param('PA_leaf1/leaf1_to_bottom1', 'Port', '1');
set_param('PA_leaf1/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_leaf1/leaf1_to_bottom1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf1/leaf1_to_bottom1', 'Tag', 'ModelerId:9cb0754d-bd10-4738-b67e-21590601f767');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf1/leaf1_to_bottom1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf1');
catch
end
save_system('PA_leaf1', fullfile(project_path, 'models/PA/PA_leaf1.slx'));
file = proj.addFile('models/PA/PA_leaf1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Top_3');
set_param('PA_Top_3', 'Tag', 'ModelerId:de7ad848-8f32-4ac7-b325-c9661dca35d4');
set_param('PA_Top_3', 'DataDictionary', 'PA_Top_3.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_3/Reference_to_leaf1');
set_param('PA_Top_3/Reference_to_leaf1', 'Tag', 'ModelerId:1e5ed3de-ecc9-4902-ba70-71eeb58c021b');
set_param('PA_Top_3/Reference_to_leaf1', 'ModelFile', 'PA_leaf1')
handles = get_param('PA_Top_3/Reference_to_leaf1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:9cb0754d-bd10-4738-b67e-21590601f767');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_3/Reference_to_leaf2');
set_param('PA_Top_3/Reference_to_leaf2', 'Tag', 'ModelerId:ee115b68-09f3-482f-9584-282bab1f6864');
set_param('PA_Top_3/Reference_to_leaf2', 'ModelFile', 'PA_leaf2')
add_block('CapellaElements/Subsystem/Function', 'PA_Top_3/container');
set_param('PA_Top_3/container', 'Tag', 'ModelerId:2485b2d7-b4db-4990-8e77-846d93b1b030');
CapSLBridge.internal.cleanSubsystem('PA_Top_3/container');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Top_3/container/bottom2_to_leaf2');
set_param('PA_Top_3/container/bottom2_to_leaf2', 'Port', '1');
set_param('PA_Top_3/container/bottom2_to_leaf2', 'OutDataTypeStr', 'Bus: busFE_bottom2_to_leaf2');
set_param('PA_Top_3/container/bottom2_to_leaf2', 'BusOutputAsStruct', 'on');
set_param('PA_Top_3/container/bottom2_to_leaf2', 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
add_block('simulink/Ports & Subsystems/In1', 'PA_Top_3/container/leaf1_to_bottom1');
set_param('PA_Top_3/container/leaf1_to_bottom1', 'Port', '1');
set_param('PA_Top_3/container/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_Top_3/container/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_3/container/Reference_to_bottom2');
set_param('PA_Top_3/container/Reference_to_bottom2', 'Tag', 'ModelerId:ffa504fd-8e29-4095-8e15-dd41f7e08583');
set_param('PA_Top_3/container/Reference_to_bottom2', 'ModelFile', 'PA_bottom2')
handles = get_param('PA_Top_3/container/Reference_to_bottom2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_Top_3/container/sub');
set_param('PA_Top_3/container/sub', 'Tag', 'ModelerId:1173662b-cfe4-4a2f-9e14-ec1b4d3d7193');
CapSLBridge.internal.cleanSubsystem('PA_Top_3/container/sub');
add_block('simulink/Ports & Subsystems/In1', 'PA_Top_3/container/sub/leaf1_to_bottom1');
set_param('PA_Top_3/container/sub/leaf1_to_bottom1', 'Port', '1');
set_param('PA_Top_3/container/sub/leaf1_to_bottom1', 'OutDataTypeStr', 'Bus: busFE_leaf1_to_bottom1');
set_param('PA_Top_3/container/sub/leaf1_to_bottom1', 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
add_block('simulink/Ports & Subsystems/Out1', 'PA_Top_3/container/sub/sub');
set_param('PA_Top_3/container/sub/sub', 'Port', '1');
set_param('PA_Top_3/container/sub/sub', 'OutDataTypeStr', 'Bus: busFE_sub');
set_param('PA_Top_3/container/sub/sub', 'BusOutputAsStruct', 'on');
set_param('PA_Top_3/container/sub/sub', 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_3/container/sub/Reference_to_bottom1');
set_param('PA_Top_3/container/sub/Reference_to_bottom1', 'Tag', 'ModelerId:0d18653e-f4cb-4b57-9d99-6fa587a4634f');
set_param('PA_Top_3/container/sub/Reference_to_bottom1', 'ModelFile', 'PA_bottom1')
handles = get_param('PA_Top_3/container/sub/Reference_to_bottom1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
clear handles;
handles = get_param('PA_Top_3/container/sub/leaf1_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container/sub/Reference_to_bottom1', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3/container/sub', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
clear line handles source target;
handles = get_param('PA_Top_3/container/sub/Reference_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container/sub/sub', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3/container/sub', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub');
set_param( line, 'Tag', 'ModelerId:395722eb-7184-475d-81dc-21e78f051c2b');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Top_3/container/sub');
catch
end
handles = get_param('PA_Top_3/container/Reference_to_bottom2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container/bottom2_to_leaf2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'bottom2_to_leaf2');
set_param( line, 'Tag', 'ModelerId:9de1a4fc-6543-4647-8fd3-60098c5dabe0');
clear line handles source target;
handles = get_param('PA_Top_3/container/leaf1_to_bottom1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container/sub', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:1c92e414-4dc0-445a-9cfa-419fa214911f');
clear line handles source target;
handles = get_param('PA_Top_3/container/sub', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container/Reference_to_bottom2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'sub');
set_param( line, 'Tag', 'ModelerId:a06d23f6-6296-4637-93b3-1780ead25ffc');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Top_3/container');
catch
end
handles = get_param('PA_Top_3/container', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/Reference_to_leaf2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'bottom2_to_leaf2');
set_param( line, 'Tag', 'ModelerId:48620e46-8811-4368-99a4-4d6401053a1e');
clear line handles source target;
handles = get_param('PA_Top_3/Reference_to_leaf1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_3/container', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_3', source, target, 'autorouting','smart');
set_param(line, 'Name', 'leaf1_to_bottom1');
set_param( line, 'Tag', 'ModelerId:f29a19c0-7886-4b9b-9ff0-df0bb36dbcb8');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Top_3');
catch
end
save_system('PA_Top_3', fullfile(project_path, 'models/PA/PA_Top_3.slx'));
file = proj.addFile('models/PA/PA_Top_3.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
