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
busFE_Top3_FE = Simulink.Bus;
busFE_Top3_FE.Description = 'ModelerId:abc2526e-4e13-4100-9db0-4dd9d6ed877a';
assignin('base', 'busFE_Top3_FE', busFE_Top3_FE);
clear elems;
if isfile('data/SA/SA_FunctionalExchanges_Categories.sldd')
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/SA/SA_FunctionalExchanges_Categories.sldd');
else
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/SA/SA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(SA_FunctionalExchanges_Categories, 'varList', {'busFE_Top3_FE'}, 'existingVarsAction','overwrite');
saveChanges(SA_FunctionalExchanges_Categories);
clear SA_FunctionalExchanges_Categories;
file = proj.addFile('data/SA/SA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_Top3.sldd')
	SA_Top3 = Simulink.data.dictionary.open('data/SA/SA_Top3.sldd');
else
	SA_Top3 = Simulink.data.dictionary.create('data/SA/SA_Top3.sldd');
end
addDataSource(SA_Top3, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_Top3);
clear SA_Top3;
file = proj.addFile('data/SA/SA_Top3.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_leaf2.sldd')
	SA_leaf2 = Simulink.data.dictionary.open('data/SA/SA_leaf2.sldd');
else
	SA_leaf2 = Simulink.data.dictionary.create('data/SA/SA_leaf2.sldd');
end
addDataSource(SA_leaf2, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_leaf2);
clear SA_leaf2;
file = proj.addFile('data/SA/SA_leaf2.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_leaf1.sldd')
	SA_leaf1 = Simulink.data.dictionary.open('data/SA/SA_leaf1.sldd');
else
	SA_leaf1 = Simulink.data.dictionary.create('data/SA/SA_leaf1.sldd');
end
addDataSource(SA_leaf1, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_leaf1);
clear SA_leaf1;
file = proj.addFile('data/SA/SA_leaf1.sldd');
addLabel(file, 'Capella Layer', 'SA');
evalin('base', "clear 'busFE_Top3_FE'");
new_system('SA_leaf1');
set_param('SA_leaf1', 'Tag', 'ModelerId:04bdd067-add9-4cac-a7fa-f96c7f970dd7');
set_param('SA_leaf1', 'DataDictionary', 'SA_leaf1.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'SA_leaf1/Top3_FE');
set_param('SA_leaf1/Top3_FE', 'Port', '1');
set_param('SA_leaf1/Top3_FE', 'OutDataTypeStr', 'Bus: busFE_Top3_FE');
set_param('SA_leaf1/Top3_FE', 'BusOutputAsStruct', 'on');
set_param('SA_leaf1/Top3_FE', 'Tag', 'ModelerId:33e462c5-a19d-4381-a62f-19968e28a819');
g_Block = add_block('simulink/Sources/Ground', 'SA_leaf1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('SA_leaf1/Top3_FE', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('SA_leaf1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('SA_leaf1');
catch
end
save_system('SA_leaf1', fullfile(project_path, 'models/SA/SA_leaf1.slx'));
file = proj.addFile('models/SA/SA_leaf1.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_leaf2');
set_param('SA_leaf2', 'Tag', 'ModelerId:48d0665e-6da4-4fea-9a2c-b8352e052536');
set_param('SA_leaf2', 'DataDictionary', 'SA_leaf2.sldd');
add_block('simulink/Ports & Subsystems/In1', 'SA_leaf2/Top3_FE');
set_param('SA_leaf2/Top3_FE', 'Port', '1');
set_param('SA_leaf2/Top3_FE', 'OutDataTypeStr', 'Bus: busFE_Top3_FE');
set_param('SA_leaf2/Top3_FE', 'Tag', 'ModelerId:84399c7b-f0b3-47c2-b268-5dab0caca84b');
t_Block = add_block('simulink/Sinks/Terminator', 'SA_leaf2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('SA_leaf2/Top3_FE', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('SA_leaf2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('SA_leaf2');
catch
end
save_system('SA_leaf2', fullfile(project_path, 'models/SA/SA_leaf2.slx'));
file = proj.addFile('models/SA/SA_leaf2.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_Top3');
set_param('SA_Top3', 'Tag', 'ModelerId:7994c9c3-8ad5-40af-9274-34790ef80f40');
set_param('SA_Top3', 'DataDictionary', 'SA_Top3.sldd');
add_block('CapellaElements/Subsystem/Function', 'SA_Top3/container');
set_param('SA_Top3/container', 'Tag', 'ModelerId:8260089b-12f0-459c-8b5e-bb1f95b1b3dd');
CapSLBridge.internal.cleanSubsystem('SA_Top3/container');
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top3/container/Reference_to_leaf2');
set_param('SA_Top3/container/Reference_to_leaf2', 'Tag', 'ModelerId:48d0665e-6da4-4fea-9a2c-b8352e052536');
set_param('SA_Top3/container/Reference_to_leaf2', 'ModelFile', 'SA_leaf2')
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top3/container/Reference_to_leaf1');
set_param('SA_Top3/container/Reference_to_leaf1', 'Tag', 'ModelerId:04bdd067-add9-4cac-a7fa-f96c7f970dd7');
set_param('SA_Top3/container/Reference_to_leaf1', 'ModelFile', 'SA_leaf1')
handles = get_param('SA_Top3/container/Reference_to_leaf1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:33e462c5-a19d-4381-a62f-19968e28a819');
clear handles;
handles = get_param('SA_Top3/container/Reference_to_leaf1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('SA_Top3/container/Reference_to_leaf2', 'PortHandles');
target = handles.Inport(1);
line = add_line('SA_Top3/container', source, target, 'autorouting','smart');
set_param(line, 'Name', 'Top3_FE');
set_param( line, 'Tag', 'ModelerId:abc2526e-4e13-4100-9db0-4dd9d6ed877a');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('SA_Top3/container');
catch
end
try
	Simulink.BlockDiagram.arrangeSystem('SA_Top3');
catch
end
save_system('SA_Top3', fullfile(project_path, 'models/SA/SA_Top3.slx'));
file = proj.addFile('models/SA/SA_Top3.slx');
addLabel(file, 'Capella Layer', 'SA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
