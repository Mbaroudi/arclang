import { PrismaClient } from '@prisma/client';
import * as crypto from 'crypto';

const prisma = new PrismaClient();

function hashPassword(password: string): string {
  return crypto.createHash('sha256').update(password).digest('hex');
}

async function main() {
  const password = hashPassword('test123');

  const user = await prisma.user.upsert({
    where: { email: 'test@arclang.dev' },
    update: {},
    create: {
      email: 'test@arclang.dev',
      name: 'Test User',
      passwordHash: password,
      role: 'USER',
    },
  });

  console.log('✅ Dev user created/updated:');
  console.log('   Email: test@arclang.dev');
  console.log('   Password: test123');
  console.log('   ID:', user.id);
}

main()
  .catch((e) => {
    console.error(e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
