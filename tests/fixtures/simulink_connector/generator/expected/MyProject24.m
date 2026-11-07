if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'mystate');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'mystate', 'Folder', project_path);
end
cd(project_path);
new_system('LA_2________10________20________30________40________50_');
add_block('simulink/Ports & Subsystems/In1', 'LA_2________10________20________30________40________50_/FunctionalExchange_1');
set_param('LA_2________10________20________30________40________50_/FunctionalExchange_1', 'Port', '1');
t_Block = add_block('simulink/Sinks/Terminator', 'LA_2________10________20________30________40________50_/Terminator', 'MakeNameUnique', 'on' );
t_handles = get_param(t_Block, 'PortHandles');
p_Block = get_param('LA_2________10________20________30________40________50_/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
terminator = t_handles.Inport(1);
port = p_handles.Outport(1);
add_line('LA_2________10________20________30________40________50_', port, terminator);
clear t_Block t_handles p_Block p_handles terminator port
try
	Simulink.BlockDiagram.arrangeSystem('LA_2________10________20________30________40________50_');
catch
end
save_system('LA_2________10________20________30________40________50_', fullfile(project_path, 'models/LA/LA_2________10________20________30________40________50_.slx'));
file = proj.addFile('models/LA/LA_2________10________20________30________40________50_.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA_1________10________20________30________40________50_');
add_block('simulink/Ports & Subsystems/Out1', 'LA_1________10________20________30________40________50_/FunctionalExchange_1');
set_param('LA_1________10________20________30________40________50_/FunctionalExchange_1', 'Port', '1');
g_Block = add_block('simulink/Sources/Ground', 'LA_1________10________20________30________40________50_/Ground', 'MakeNameUnique', 'on' );
g_handles = get_param(g_Block, 'PortHandles');
p_Block = get_param('LA_1________10________20________30________40________50_/FunctionalExchange_1', 'handle');
p_handles = get_param(p_Block, 'PortHandles');
ground = g_handles.Outport(1);
port = p_handles.Inport(1);
add_line('LA_1________10________20________30________40________50_', ground, port);
clear g_Block g_handles p_Block p_handles ground port
try
	Simulink.BlockDiagram.arrangeSystem('LA_1________10________20________30________40________50_');
catch
end
save_system('LA_1________10________20________30________40________50_', fullfile(project_path, 'models/LA/LA_1________10________20________30________40________50_.slx'));
file = proj.addFile('models/LA/LA_1________10________20________30________40________50_.slx');
addLabel(file, 'Capella Layer', 'LA');
new_system('LA________10________20________30________40________50');
add_block('CapellaElements/ModelRef/Model_Function', 'LA________10________20________30________40________50/Reference_to_LogicalFunction_1');
set_param('LA________10________20________30________40________50/Reference_to_LogicalFunction_1', 'ModelFile', 'LA_1________10________20________30________40________50_')
handles = get_param('LA________10________20________30________40________50/Reference_to_LogicalFunction_1', 'porthandles');
clear handles;
add_block('CapellaElements/ModelRef/Model_Function', 'LA________10________20________30________40________50/Reference_to_LogicalFunction_2');
set_param('LA________10________20________30________40________50/Reference_to_LogicalFunction_2', 'ModelFile', 'LA_2________10________20________30________40________50_')
handles = get_param('LA________10________20________30________40________50/Reference_to_LogicalFunction_1', 'PortHandles');
source = handles.Outport(1);
handles = get_param('LA________10________20________30________40________50/Reference_to_LogicalFunction_2', 'PortHandles');
target = handles.Inport(1);
line = add_line('LA________10________20________30________40________50', source, target, 'autorouting','smart');
set_param(line, 'Name', 'FunctionalExchange_1');
clear line handles source target;
try
	Simulink.BlockDiagram.arrangeSystem('LA________10________20________30________40________50');
catch
end
save_system('LA________10________20________30________40________50', fullfile(project_path, 'models/LA/LA________10________20________30________40________50.slx'));
file = proj.addFile('models/LA/LA________10________20________30________40________50.slx');
addLabel(file, 'Capella Layer', 'LA');
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
