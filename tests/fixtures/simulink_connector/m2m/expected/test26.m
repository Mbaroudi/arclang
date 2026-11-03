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
busFE_FunctionalExchange_1 = Simulink.Bus;
busFE_FunctionalExchange_1.Description = 'ModelerId:6393c531-9710-426e-aa29-289e5936e2ed';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
if isfile('data/SA/SA_FunctionalExchanges_Categories.sldd')
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/SA/SA_FunctionalExchanges_Categories.sldd');
else
	SA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/SA/SA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(SA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
saveChanges(SA_FunctionalExchanges_Categories);
clear SA_FunctionalExchanges_Categories;
file = proj.addFile('data/SA/SA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_SystemFunction_1.sldd')
	SA_SystemFunction_1 = Simulink.data.dictionary.open('data/SA/SA_SystemFunction_1.sldd');
else
	SA_SystemFunction_1 = Simulink.data.dictionary.create('data/SA/SA_SystemFunction_1.sldd');
end
addDataSource(SA_SystemFunction_1, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_SystemFunction_1);
clear SA_SystemFunction_1;
file = proj.addFile('data/SA/SA_SystemFunction_1.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_SystemFunction_2.sldd')
	SA_SystemFunction_2 = Simulink.data.dictionary.open('data/SA/SA_SystemFunction_2.sldd');
else
	SA_SystemFunction_2 = Simulink.data.dictionary.create('data/SA/SA_SystemFunction_2.sldd');
end
addDataSource(SA_SystemFunction_2, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_SystemFunction_2);
clear SA_SystemFunction_2;
file = proj.addFile('data/SA/SA_SystemFunction_2.sldd');
addLabel(file, 'Capella Layer', 'SA');
if isfile('data/SA/SA_Top4.sldd')
	SA_Top4 = Simulink.data.dictionary.open('data/SA/SA_Top4.sldd');
else
	SA_Top4 = Simulink.data.dictionary.create('data/SA/SA_Top4.sldd');
end
addDataSource(SA_Top4, 'SA_FunctionalExchanges_Categories.sldd');
saveChanges(SA_Top4);
clear SA_Top4;
file = proj.addFile('data/SA/SA_Top4.sldd');
addLabel(file, 'Capella Layer', 'SA');
evalin('base', "clear 'busFE_FunctionalExchange_1'");
new_system('SA_SystemFunction_2');
set_param('SA_SystemFunction_2', 'Tag', 'ModelerId:7e9d1d1d-d670-46c6-a1d7-802a52d02c41');
set_param('SA_SystemFunction_2', 'DataDictionary', 'SA_SystemFunction_2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'SA_SystemFunction_2/FunctionalExchange_1');
set_param('SA_SystemFunction_2/FunctionalExchange_1', 'Port', '1');
set_param('SA_SystemFunction_2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('SA_SystemFunction_2/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('SA_SystemFunction_2/FunctionalExchange_1', 'Tag', 'ModelerId:fbcfa67f-59dc-4820-a7c2-b616a006e020');
g_Block = add_block('simulink/Sources/Ground', 'SA_SystemFunction_2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('SA_SystemFunction_2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('SA_SystemFunction_2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('SA_SystemFunction_2');
catch
end
save_system('SA_SystemFunction_2', fullfile(project_path, 'models/SA/SA_SystemFunction_2.slx'));
file = proj.addFile('models/SA/SA_SystemFunction_2.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_SystemFunction_1');
set_param('SA_SystemFunction_1', 'Tag', 'ModelerId:1020dde1-3ffb-48e7-9154-a72c55cba581');
set_param('SA_SystemFunction_1', 'DataDictionary', 'SA_SystemFunction_1.sldd');
add_block('simulink/Ports & Subsystems/In1', 'SA_SystemFunction_1/FunctionalExchange_1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'Port', '1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('SA_SystemFunction_1/FunctionalExchange_1', 'Tag', 'ModelerId:2a2ffdcf-97f1-4ef8-94ef-cd9a73376ed7');
t_Block = add_block('simulink/Sinks/Terminator', 'SA_SystemFunction_1/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('SA_SystemFunction_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('SA_SystemFunction_1', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('SA_SystemFunction_1');
catch
end
save_system('SA_SystemFunction_1', fullfile(project_path, 'models/SA/SA_SystemFunction_1.slx'));
file = proj.addFile('models/SA/SA_SystemFunction_1.slx');
addLabel(file, 'Capella Layer', 'SA');
new_system('SA_Top4');
set_param('SA_Top4', 'Tag', 'ModelerId:ff0a0149-1a00-4b77-916d-91df4ac52de3');
set_param('SA_Top4', 'DataDictionary', 'SA_Top4.sldd');
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top4/Reference_to_SystemFunction_1');
set_param('SA_Top4/Reference_to_SystemFunction_1', 'Tag', 'ModelerId:1020dde1-3ffb-48e7-9154-a72c55cba581');
set_param('SA_Top4/Reference_to_SystemFunction_1', 'ModelFile', 'SA_SystemFunction_1')
add_block('CapellaElements/ModelRef/Model_Function', 'SA_Top4/Reference_to_SystemFunction_2');
set_param('SA_Top4/Reference_to_SystemFunction_2', 'Tag', 'ModelerId:7e9d1d1d-d670-46c6-a1d7-802a52d02c41');
set_param('SA_Top4/Reference_to_SystemFunction_2', 'ModelFile', 'SA_SystemFunction_2')
handles = get_param('SA_Top4/Reference_to_SystemFunction_2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:fbcfa67f-59dc-4820-a7c2-b616a006e020');
clear handles;
handles = get_param('SA_Top4/Reference_to_SystemFunction_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('SA_Top4/Reference_to_SystemFunction_1', 'PortHandles');
target = handles.Inport(1);
line = add_line('SA_Top4', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:6393c531-9710-426e-aa29-289e5936e2ed');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('SA_Top4');
catch
end
save_system('SA_Top4', fullfile(project_path, 'models/SA/SA_Top4.slx'));
file = proj.addFile('models/SA/SA_Top4.slx');
addLabel(file, 'Capella Layer', 'SA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
