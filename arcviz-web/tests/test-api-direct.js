const axios = require('axios');

const API_URL = 'http://localhost:4000/api';

async function testAPI() {
  try {
    // Register a user
    const email = `test${Date.now()}@test.com`;
    const registerRes = await axios.post(`${API_URL}/auth/register`, {
      email,
      password: 'Test123!',
      name: 'Test User'
    });
    
    const token = registerRes.data.token;
    console.log('✓ Registered user, got token');
    
    // Compile code
    const code = `model Test {
    metadata {
        name: "Test"
    }
    requirements system {
        req SYS-001 "Test Requirement" {
            description: "test"
        }
    }
    architecture logical {
        component TestComp "Test Component" {
            description: "test component"
        }
    }
}`;
    
    const compileRes = await axios.post(`${API_URL}/compile`, 
      { code },
      { headers: { Authorization: `Bearer ${token}` } }
    );
    
    console.log('✓ Compilation response:', JSON.stringify(compileRes.data, null, 2));
    console.log(`\n✓ Diagram has ${compileRes.data.diagram?.nodes?.length || 0} nodes and ${compileRes.data.diagram?.edges?.length || 0} edges`);
    
  } catch (error) {
    console.error('✗ Error:', error.response?.data || error.message);
  }
}

testAPI();
