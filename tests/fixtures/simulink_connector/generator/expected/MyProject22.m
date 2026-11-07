if not(exist('project_path', 'var'))
	project_path = userpath;
	project_path =  fullfile(project_path, 'MyProject22');
end
if isfolder(project_path)
	proj = slproject.loadProject(project_path);
else
	proj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', 'MyProject22', 'Folder', project_path);
end
cd(project_path);
PhysicalQuantity_1 = Simulink.Signal;
PhysicalQuantity_1.Unit = 'm';
assignin('base', 'PhysicalQuantity_1', PhysicalQuantity_1);
clear elems;
elems(1) = Simulink.BusElement;
elems(1).Name = 'Property_1';
busClass_Class_1 = Simulink.Bus;
busClass_Class_1.Elements = elems;
assignin('base', 'busClass_Class_1', busClass_Class_1);
clear elems;
if isfile('data/PA/PA_Data.sldd')
	PA_Data = Simulink.data.dictionary.open('data/PA/PA_Data.sldd');
else
	PA_Data = Simulink.data.dictionary.create('data/PA/PA_Data.sldd');
end
importFromBaseWorkspace(PA_Data, 'varList', {'PhysicalQuantity_1', 'busClass_Class_1'}, 'existingVarsAction','overwrite');
saveChanges(PA_Data);
clear PA_Data;
file = proj.addFile('data/PA/PA_Data.sldd');
addLabel(file, 'Capella Layer', 'PA');
evalin('base', "clear 'PhysicalQuantity_1' 'busClass_Class_1'");
cd(proj.Information.ProjectStartupFolder);
clear proj file project_path;
