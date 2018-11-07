using Microsoft.VisualStudio.TestTools.UnitTesting;
using System.Linq;
using System.Threading;

namespace YouthCenterSignIn.Logic.Tests
{
    [TestClass]
    public class PersonSearchTests : TestBase
    {
        [TestMethod]
        public void PersonSearch_ResultsTest()
        {
            var search = new PersonSearch
            {
                SearchText = "James"
            };

            Thread.Sleep(1);
            Assert.AreEqual("James", search.Single().FirstName,
                "The search text was set to James, so the first (and only) result should have a first name of James.");
        }
    }
}
