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
busFE_FunctionalExchange_1.Description = 'ModelerId:6e911035-42a1-434b-b1c3-f9c696de00e8';
assignin('base', 'busFE_FunctionalExchange_1', busFE_FunctionalExchange_1);
clear elems;
if isfile('data/PA/PA_FunctionalExchanges_Categories.sldd')
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.open('data/PA/PA_FunctionalExchanges_Categories.sldd');
else
	PA_FunctionalExchanges_Categories = Simulink.data.dictionary.create('data/PA/PA_FunctionalExchanges_Categories.sldd');
end
importFromBaseWorkspace(PA_FunctionalExchanges_Categories, 'varList', {'busFE_FunctionalExchange_1'}, 'existingVarsAction','overwrite');
saveChanges(PA_FunctionalExchanges_Categories);
clear PA_FunctionalExchanges_Categories;
file = proj.addFile('data/PA/PA_FunctionalExchanges_Categories.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_pf_1.sldd')
	PA_pf_1 = Simulink.data.dictionary.open('data/PA/PA_pf_1.sldd');
else
	PA_pf_1 = Simulink.data.dictionary.create('data/PA/PA_pf_1.sldd');
end
addDataSource(PA_pf_1, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_pf_1);
clear PA_pf_1;
file = proj.addFile('data/PA/PA_pf_1.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_pf_2.sldd')
	PA_pf_2 = Simulink.data.dictionary.open('data/PA/PA_pf_2.sldd');
else
	PA_pf_2 = Simulink.data.dictionary.create('data/PA/PA_pf_2.sldd');
end
addDataSource(PA_pf_2, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_pf_2);
clear PA_pf_2;
file = proj.addFile('data/PA/PA_pf_2.sldd');
addLabel(file, 'Capella Layer', 'PA');
if isfile('data/PA/PA_Top_6.sldd')
	PA_Top_6 = Simulink.data.dictionary.open('data/PA/PA_Top_6.sldd');
else
	PA_Top_6 = Simulink.data.dictionary.create('data/PA/PA_Top_6.sldd');
end
addDataSource(PA_Top_6, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_Top_6);
clear PA_Top_6;
file = proj.addFile('data/PA/PA_Top_6.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'busFE_FunctionalExchange_1'");
new_system('PA_pf_2');
set_param('PA_pf_2', 'Tag', 'ModelerId:093e62cb-9bb3-4141-8507-4229af11d8e0');
set_param('PA_pf_2', 'DataDictionary', 'PA_pf_2.sldd');
add_block('simulink/Ports & Subsystems/Enable', 'PA_pf_2/EN_pf_2');
set_param('PA_pf_2/EN_pf_2', 'Description', 'State 1');
set_param('PA_pf_2/EN_pf_2', 'AttributesFormatString', 'Available in Modes & States: %<Description>')
set_param('PA_pf_2/EN_pf_2', 'ShowName', 'off')
add_block('simulink/Ports & Subsystems/In1', 'PA_pf_2/FunctionalExchange_1');
set_param('PA_pf_2/FunctionalExchange_1', 'Port', '1');
set_param('PA_pf_2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_pf_2/FunctionalExchange_1', 'Tag', 'ModelerId:62831711-028f-41ec-9dce-5a036630b9c4');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_pf_2/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_pf_2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_pf_2', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_pf_2');
catch
end
save_system('PA_pf_2', fullfile(project_path, 'models/PA/PA_pf_2.slx'));
file = proj.addFile('models/PA/PA_pf_2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_pf_1');
set_param('PA_pf_1', 'Tag', 'ModelerId:36ece6e6-ae73-47b2-b9b3-9e102f6cee51');
set_param('PA_pf_1', 'DataDictionary', 'PA_pf_1.sldd');
add_block('simulink/Ports & Subsystems/Enable', 'PA_pf_1/EN_pf_1');
set_param('PA_pf_1/EN_pf_1', 'Description', 'State 1');
set_param('PA_pf_1/EN_pf_1', 'AttributesFormatString', 'Available in Modes & States: %<Description>')
set_param('PA_pf_1/EN_pf_1', 'ShowName', 'off')
add_block('simulink/Ports & Subsystems/Out1', 'PA_pf_1/FunctionalExchange_1');
set_param('PA_pf_1/FunctionalExchange_1', 'Port', '1');
set_param('PA_pf_1/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_pf_1/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_pf_1/FunctionalExchange_1', 'Tag', 'ModelerId:d12d6e5e-326b-40f5-9dbf-8b98a7511d75');
g_Block = add_block('simulink/Sources/Ground', 'PA_pf_1/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_pf_1/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_pf_1', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_pf_1');
catch
end
save_system('PA_pf_1', fullfile(project_path, 'models/PA/PA_pf_1.slx'));
file = proj.addFile('models/PA/PA_pf_1.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_Top_6');
set_param('PA_Top_6', 'Tag', 'ModelerId:26272849-2349-45f9-9e8b-80b131e5d506');
set_param('PA_Top_6', 'DataDictionary', 'PA_Top_6.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_Top_6/EN_pf_1');
set_param('PA_Top_6/EN_pf_1', 'Port', '1');
add_block('simulink/Ports & Subsystems/In1', 'PA_Top_6/EN_pf_2');
set_param('PA_Top_6/EN_pf_2', 'Port', '2');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_6/Reference_to_pf_1');
set_param('PA_Top_6/Reference_to_pf_1', 'Tag', 'ModelerId:36ece6e6-ae73-47b2-b9b3-9e102f6cee51');
set_param('PA_Top_6/Reference_to_pf_1', 'ModelFile', 'PA_pf_1')
handles = get_param('PA_Top_6/Reference_to_pf_1', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:d12d6e5e-326b-40f5-9dbf-8b98a7511d75');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'PA_Top_6/Reference_to_pf_2');
set_param('PA_Top_6/Reference_to_pf_2', 'Tag', 'ModelerId:093e62cb-9bb3-4141-8507-4229af11d8e0');
set_param('PA_Top_6/Reference_to_pf_2', 'ModelFile', 'PA_pf_2')
handles = get_param('PA_Top_6/EN_pf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_6/Reference_to_pf_1', 'PortHandles');
target = handles.Enable(1);
line = add_line('PA_Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_1');
clear line handles source target;
handles = get_param('PA_Top_6/EN_pf_2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_6/Reference_to_pf_2', 'PortHandles');
target = handles.Enable(1);
line = add_line('PA_Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'EN_pf_2');
clear line handles source target;
handles = get_param('PA_Top_6/Reference_to_pf_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_Top_6/Reference_to_pf_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_Top_6', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:6e911035-42a1-434b-b1c3-f9c696de00e8');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_Top_6');
catch
end
save_system('PA_Top_6', fullfile(project_path, 'models/PA/PA_Top_6.slx'));
file = proj.addFile('models/PA/PA_Top_6.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
