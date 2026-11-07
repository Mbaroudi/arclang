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
busFE_FunctionalExchange_1.Description = 'ModelerId:8e52be06-ff6f-4fb2-b496-571d8ac36692';
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
if isfile('data/PA/PA_container.sldd')
	PA_container = Simulink.data.dictionary.open('data/PA/PA_container.sldd');
else
	PA_container = Simulink.data.dictionary.create('data/PA/PA_container.sldd');
end
addDataSource(PA_container, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_container);
clear PA_container;
file = proj.addFile('data/PA/PA_container.sldd');
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
if isfile('data/PA/PA_leaf3.sldd')
	PA_leaf3 = Simulink.data.dictionary.open('data/PA/PA_leaf3.sldd');
else
	PA_leaf3 = Simulink.data.dictionary.create('data/PA/PA_leaf3.sldd');
end
addDataSource(PA_leaf3, 'PA_FunctionalExchanges_Categories.sldd');
saveChanges(PA_leaf3);
clear PA_leaf3;
file = proj.addFile('data/PA/PA_leaf3.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'busFE_FunctionalExchange_1'");
new_system('PA_leaf3');
set_param('PA_leaf3', 'Tag', 'ModelerId:03f7e903-d08f-4375-8c0f-964f1ed50ec5');
set_param('PA_leaf3', 'DataDictionary', 'PA_leaf3.sldd');
add_block('simulink/Ports & Subsystems/In1', 'PA_leaf3/FunctionalExchange_1');
set_param('PA_leaf3/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf3/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf3/FunctionalExchange_1', 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
t_Block = add_block('simulink/Sinks/Terminator', 'PA_leaf3/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('PA_leaf3/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('PA_leaf3', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf3');
catch
end
save_system('PA_leaf3', fullfile(project_path, 'models/PA/PA_leaf3.slx'));
file = proj.addFile('models/PA/PA_leaf3.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_leaf2');
set_param('PA_leaf2', 'Tag', 'ModelerId:82c22b93-0695-45f9-a80e-07244eaced6b');
set_param('PA_leaf2', 'DataDictionary', 'PA_leaf2.sldd');
add_block('simulink/Ports & Subsystems/Out1', 'PA_leaf2/FunctionalExchange_1');
set_param('PA_leaf2/FunctionalExchange_1', 'Port', '1');
set_param('PA_leaf2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_leaf2/FunctionalExchange_1', 'BusOutputAsStruct', 'on');
set_param('PA_leaf2/FunctionalExchange_1', 'Tag', 'ModelerId:74707e01-4261-4020-ad5e-b0cefc568ed0');
g_Block = add_block('simulink/Sources/Ground', 'PA_leaf2/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('PA_leaf2/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('PA_leaf2', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('PA_leaf2');
catch
end
save_system('PA_leaf2', fullfile(project_path, 'models/PA/PA_leaf2.slx'));
file = proj.addFile('models/PA/PA_leaf2.slx');
addLabel(file, 'Capella Layer', 'PA');
new_system('PA_container');
set_param('PA_container', 'Tag', 'ModelerId:1798c7cb-3d42-48a2-be9b-2d16ef9c9b24');
set_param('PA_container', 'DataDictionary', 'PA_container.sldd');
add_block('CapellaElements/Subsystem/Function', 'PA_container/container1');
set_param('PA_container/container1', 'Tag', 'ModelerId:5d08f6cc-97e2-426e-b4c7-368d15959c1f');
CapSLBridge.internal.cleanSubsystem('PA_container/container1');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_container/container1/Reference_to_leaf2');
set_param('PA_container/container1/Reference_to_leaf2', 'Tag', 'ModelerId:82c22b93-0695-45f9-a80e-07244eaced6b');
set_param('PA_container/container1/Reference_to_leaf2', 'ModelFile', 'PA_leaf2')
handles = get_param('PA_container/container1/Reference_to_leaf2', 'porthandles');
set_param(handles.Outport(1), 'Tag', 'ModelerId:74707e01-4261-4020-ad5e-b0cefc568ed0');
clear handles;
add_block('CapellaElements/Subsystem/Function', 'PA_container/container1/container2');
set_param('PA_container/container1/container2', 'Tag', 'ModelerId:e4c9f262-55d2-4446-b865-ba7990ec8516');
CapSLBridge.internal.cleanSubsystem('PA_container/container1/container2');
add_block('simulink/Ports & Subsystems/In1', 'PA_container/container1/container2/FunctionalExchange_1');
set_param('PA_container/container1/container2/FunctionalExchange_1', 'Port', '1');
set_param('PA_container/container1/container2/FunctionalExchange_1', 'OutDataTypeStr', 'Bus: busFE_FunctionalExchange_1');
set_param('PA_container/container1/container2/FunctionalExchange_1', 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
add_block('CapellaElements/ModelRef/Model_Function', 'PA_container/container1/container2/Reference_to_leaf3');
set_param('PA_container/container1/container2/Reference_to_leaf3', 'Tag', 'ModelerId:03f7e903-d08f-4375-8c0f-964f1ed50ec5');
set_param('PA_container/container1/container2/Reference_to_leaf3', 'ModelFile', 'PA_leaf3')
handles = get_param('PA_container/container1/container2/FunctionalExchange_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container/container1/container2/Reference_to_leaf3', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container/container1/container2', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:2d99ce12-ffe0-4139-b3c6-63bbda0978ab');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_container/container1/container2');
catch
end
handles = get_param('PA_container/container1/Reference_to_leaf2', 'PortHandles');
source = handles.Outport(1);
handles = get_param('PA_container/container1/container2', 'PortHandles');
target = handles.Inport(1);
line = add_line('PA_container/container1', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
set_param( line, 'Tag', 'ModelerId:8e52be06-ff6f-4fb2-b496-571d8ac36692');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('PA_container/container1');
catch
end
try
	Simulink.BlockDiagram.arrangeSystem('PA_container');
catch
end
save_system('PA_container', fullfile(project_path, 'models/PA/PA_container.slx'));
file = proj.addFile('models/PA/PA_container.slx');
addLabel(file, 'Capella Layer', 'PA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
